// ref https://github.com/risc0/zeth/blob/main/host/src/main.rs
use std::fmt::Debug;
use bonsai_sdk::alpha::responses::SnarkReceipt;
use log::{debug, error, info, warn};
use risc0_zkvm::{
    compute_image_id,
    serde::to_vec,
    sha::{Digest, Digestible},
    Assumption, ExecutorEnv, ExecutorImpl, Receipt, Segment, SegmentRef,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

const BONSAI_API_URL: &str = "https://api.bonsai.xyz/";

pub async fn prove_bonsai<O: Eq + Debug + DeserializeOwned>(
    api_key: String,
    encoded_input: Vec<u32>,
    elf: &[u8],
    expected_output: &O,
    assumption_uuids: Vec<String>,
) -> anyhow::Result<(String, Receipt)> {
    info!("Proving on Bonsai");
    // Compute the image_id, then upload the ELF with the image_id as its key.
    let image_id = risc0_zkvm::compute_image_id(elf)?;
    let encoded_image_id = hex::encode(image_id);
    // Prepare input data
    let input_data = bytemuck::cast_slice(&encoded_input).to_vec();

    let client = bonsai_sdk::alpha_async::get_client_from_parts(BONSAI_API_URL.to_owned(), api_key.clone(), risc0_zkvm::VERSION).await?;
    client.upload_img(&encoded_image_id, elf.to_vec())?;
    // upload input
    let input_id = client.upload_input(input_data.clone())?;

    let session = client.create_session(
        encoded_image_id.clone(),
        input_id.clone(),
        assumption_uuids.clone(),
    )?;

    verify_bonsai_receipt(api_key, image_id, expected_output, session.uuid.clone(), 8).await
}

pub async fn verify_bonsai_receipt<O: Eq + Debug + DeserializeOwned>(
    api_key: String,
    image_id: Digest,
    expected_output: &O,
    uuid: String,
    max_retries: usize,
) -> anyhow::Result<(String, Receipt)> {
    info!("Tracking receipt uuid: {}", uuid);
    let session = bonsai_sdk::alpha::SessionId { uuid };

    loop {
        let mut res = None;
        for attempt in 1..=max_retries {
            let client = bonsai_sdk::alpha_async::get_client_from_parts(BONSAI_API_URL.to_owned(), api_key, risc0_zkvm::VERSION).await?;

            match session.status(&client) {
                Ok(response) => {
                    res = Some(response);
                    break;
                }
                Err(err) => {
                    if attempt == max_retries {
                        anyhow::bail!(err);
                    }
                    warn!(
                        "Attempt {}/{} for session status request: {:?}",
                        attempt, max_retries, err
                    );
                    std::thread::sleep(std::time::Duration::from_secs(15));
                    continue;
                }
            }
        }

        let res = res.unwrap();

        if res.status == "RUNNING" {
            info!(
                "Current status: {} - state: {} - continue polling...",
                res.status,
                res.state.unwrap_or_default()
            );
            std::thread::sleep(std::time::Duration::from_secs(15));
        } else if res.status == "SUCCEEDED" {
            // Download the receipt, containing the output
            let receipt_url = res
                .receipt_url
                .expect("API error, missing receipt on completed session");
            let client = bonsai_sdk::alpha_async::get_client_from_env(risc0_zkvm::VERSION).await?;
            let receipt_buf = client.download(&receipt_url)?;
            let receipt: Receipt = bincode::deserialize(&receipt_buf)?;
            receipt
                .verify(image_id)
                .expect("Receipt verification failed");
            // verify output
            let receipt_output: O = receipt.journal.decode().unwrap();
            if expected_output == &receipt_output {
                info!("Receipt validated!");
            } else {
                error!(
                    "Output mismatch! Receipt: {:?}, expected: {:?}",
                    receipt_output, expected_output,
                );
            }
            return Ok((session.uuid, receipt));
        } else {
            panic!(
                "Workflow exited: {} - | err: {}",
                res.status,
                res.error_msg.unwrap_or_default()
            );
        }
    }
}

pub async fn stark2snark(
    api_key: String,
    image_id: Digest,
    stark_uuid: String,
    stark_receipt: Receipt,
) -> anyhow::Result<(String, SnarkReceipt)> {
    info!("Submitting SNARK workload");

    // Otherwise compute on Bonsai
    let stark_uuid = upload_receipt(&stark_receipt).await?;

    let client = bonsai_sdk::alpha_async::get_client_from_env(risc0_zkvm::VERSION).await?;
    let snark_uuid = client.create_snark(stark_uuid)?;

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

pub async fn upload_receipt(api_key: String, receipt: &Receipt) -> anyhow::Result<String> {
    let client = bonsai_sdk::alpha_async::get_client_from_env(risc0_zkvm::VERSION).await?;
    Ok(client.upload_receipt(bincode::serialize(receipt)?)?)
}
