// ref https://github.com/risc0/zeth/blob/main/host/src/main.rs
use std::{fmt::Debug, time::Duration};
use anyhow::{Result, bail, Context};
use log::{debug, error, info};
use risc0_zkvm::{
    serde::to_vec, sha::{Digest, Digestible}, Assumption, ExecutorEnv, ExecutorImpl, Journal, Receipt, Segment, SegmentRef
};
use bonsai_sdk::alpha as bonsai_sdk;
use risc0_ethereum_contracts::groth16::Seal;
use ethers_core::abi::Token;
use serde::{de::DeserializeOwned, Deserialize, Serialize};


pub fn prove_bonsai(
    encoded_input: Vec<u32>,
    elf: &[u8],
    assumption_uuids: Vec<String>,
) -> anyhow::Result<(String, Receipt)> {
    info!("Proving on Bonsai");
    // Compute the image_id, then upload the ELF with the image_id as its key.
    let image_id = risc0_zkvm::compute_image_id(elf)?;
    let encoded_image_id = image_id.to_string();
    // Prepare input data
    let input_data = bytemuck::cast_slice(&encoded_input).to_vec();

    let client = bonsai_sdk::Client::from_env(risc0_zkvm::VERSION)?;
    
    client.upload_img(&encoded_image_id, elf.to_vec())?;
    info!("Image ID: 0x{}", encoded_image_id);
    // upload input
    let input_id = client.upload_input(input_data.clone())?;

    let session = client.create_session(
        encoded_image_id.clone(),
        input_id.clone(),
        assumption_uuids.clone(),
    )?;
    info!("Session ID: {}", session.uuid);
    let _receipt = loop {
        let res = session.status(&client)?;
        if res.status == "RUNNING" {
            log::info!(
                "Current status: {} - state: {} - continue polling...",
                res.status,
                res.state.unwrap_or_default()
            );
            std::thread::sleep(Duration::from_secs(15));
            continue;
        }
        if res.status == "SUCCEEDED" {
            // Download the receipt, containing the output.
            let receipt_url = res
                .receipt_url
                .context("API error, missing receipt on completed session")?;

            let receipt_buf = client.download(&receipt_url)?;
            let receipt: Receipt = bincode::deserialize(&receipt_buf)?;

            break receipt;
        }

        bail!(
            "Workflow exited: {} - | err: {}",
            res.status,
            res.error_msg.unwrap_or_default()
        );
    };
    info!("Session finished");
    Ok((session.uuid, _receipt))
}



pub fn stark2snark(
    stark_uuid: String,
    stark_receipt: Receipt,
) -> anyhow::Result<(String, bonsai_sdk::responses::SnarkReceipt)> {

    let client = bonsai_sdk::Client::from_env(risc0_zkvm::VERSION)?;
    let snark_uuid = client.create_snark(stark_uuid)?;
    info!("Submitted SNARK workload: {}", snark_uuid.uuid);
    
    let snark_receipt = loop {
        let res = snark_uuid.status(&client)?;

        if res.status == "RUNNING" {
            info!("Current status: {} - continue polling...", res.status,);
            std::thread::sleep(std::time::Duration::from_secs(15));
        } else if res.status == "SUCCEEDED" {
            break res
                .output
                .expect("Bonsai response is missing SnarkReceipt.");
        } else {
            panic!(
                "Workflow exited: {} - | err: {}",
                res.status,
                res.error_msg.unwrap_or_default()
            );
        }
    };

    let stark_psd = stark_receipt.get_claim()?.post.digest();
    let snark_psd = Digest::try_from(snark_receipt.post_state_digest.as_slice())?;

    if stark_psd != snark_psd {
        error!("SNARK/STARK Post State Digest mismatch!");
        error!("STARK: {}", hex::encode(stark_psd));
        error!("SNARK: {}", hex::encode(snark_psd));
    }

    if snark_receipt.journal != stark_receipt.journal.bytes {
        error!("SNARK/STARK Receipt Journal mismatch!");
        error!("STARK: {}", hex::encode(&stark_receipt.journal.bytes));
        error!("SNARK: {}", hex::encode(&snark_receipt.journal));
    };

    let snark_data = (snark_uuid.uuid, snark_receipt);


    Ok(snark_data)
}



/// Prove the given ELF locally with the given input and assumptions. The segments are
/// stored in a temporary directory, to allow for proofs larger than the available memory.
pub fn prove_locally(
    segment_limit_po2: u32,
    encoded_input: Vec<u32>,
    elf: &[u8],
    assumptions: Vec<Assumption>,
    profile: bool,
    profile_reference: &String,
) -> Receipt {
    debug!("Proving with segment_limit_po2 = {:?}", segment_limit_po2);
    debug!(
        "Input size: {} words ( {} MB )",
        encoded_input.len(),
        encoded_input.len() * 4 / 1_000_000
    );

    info!("Running the prover...");
    let session = {
        let mut env_builder = ExecutorEnv::builder();
        env_builder
            .session_limit(None)
            .segment_limit_po2(segment_limit_po2)
            .write_slice(&encoded_input);

        if profile {
            info!("Profiling enabled.");
            env_builder.enable_profiler(format!("profile_{}.pb", profile_reference));
        }

        for assumption in assumptions {
            env_builder.add_assumption(assumption);
        }

        let env = env_builder.build().unwrap();
        let mut exec = ExecutorImpl::from_elf(env, elf).unwrap();
        exec.run().unwrap()
    };
    session.prove().unwrap()
}


pub fn prove<I: Serialize>(
    input: &I,
    elf: &[u8],
    use_bonsai: bool,
    assumptions: (Vec<Assumption>, Vec<String>),
) -> Result<(String, Receipt)> {

    let (assumption_instances, assumption_uuids) = assumptions;
    let encoded_input = to_vec(input).expect("Could not serialize proving input!");


    let (receipt_uuid, receipt) =
        if use_bonsai {
            prove_bonsai(encoded_input.clone(),elf, assumption_uuids.clone())?
        } else {
            // run prover
            (
                Default::default(),
                prove_locally(
                    20,
                    encoded_input,
                    elf,
                    assumption_instances,
                    false,
                    &Default::default(),
                ),
            )
        };

    let result = (receipt_uuid, receipt);
    // return result
    Ok(result)
}

const NULL_SEGMENT_REF: NullSegmentRef = NullSegmentRef {};
#[derive(Serialize, Deserialize)]
struct NullSegmentRef {}

impl SegmentRef for NullSegmentRef {
    fn resolve(&self) -> anyhow::Result<Segment> {
        unimplemented!()
    }
}

/// Execute the guest code with the given input and verify the output.
pub fn execute<T: Serialize>(
    input: &T,
    segment_limit_po2: u32,
    profile: bool,
    elf: &[u8],
    profile_reference: &String,
) -> Journal {
    debug!(
        "Running in executor with segment_limit_po2 = {:?}",
        segment_limit_po2
    );

    let input = to_vec(input).expect("Could not serialize input!");
    debug!(
        "Input size: {} words ( {} MB )",
        input.len(),
        input.len() * 4 / 1_000_000
    );

    info!("Running the executor...");
    let session = {
        let mut env_builder = ExecutorEnv::builder();
        env_builder
            .session_limit(None)
            .segment_limit_po2(segment_limit_po2)
            .write_slice(&input);

        if profile {
            info!("Profiling enabled.");
            env_builder.enable_profiler(format!("profile_{}.pb", profile_reference));
        }

        let env = env_builder.build().unwrap();
        let mut exec = ExecutorImpl::from_elf(env, elf).unwrap();

        exec.run_with_callback(|_| Ok(Box::new(NULL_SEGMENT_REF)))
            .unwrap()
    };
    println!(
        "Executor ran in (roughly) {} cycles",
        session.segments.len() * (1 << segment_limit_po2)
    );
    // verify output
    session.journal.unwrap()
}


pub fn groth16_snark_encode(snark_receipt: bonsai_sdk::responses::SnarkReceipt) -> Result<Vec<u8>> {
    let seal = Seal::abi_encode(snark_receipt.snark)?;
    let output_tokens = vec![
        Token::Bytes(snark_receipt.journal),
        Token::FixedBytes(snark_receipt.post_state_digest),
        Token::Bytes(seal),
    ];
    let output = ethers_core::abi::encode(&output_tokens);
    Ok(output)
}
