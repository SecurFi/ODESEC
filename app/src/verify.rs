use std::fs::File;

use bridge::VmOutput;
use chains_evm_core::{evm_primitives::ToAlloy, provider::try_get_http_provider};
use ethers_providers::Middleware;
use clap::Parser;
use clio::Output;
use anyhow::{bail, Result};

use crate::proof::Proof;
use guests::EXPLOIT_ID;


#[derive(Parser, Debug)]
pub struct VerifyArgs {
    
    path: String,

    #[clap(short, long)]
    rpc_url: String,

    /// Output file 
    #[clap(long, short, value_parser, default_value = "-")]
    output: Output,

}


impl VerifyArgs {
    pub async fn run(self) -> Result<()> {
        let file = File::open(self.path)?;
        let proof = Proof::load(file)?;
        proof.receipt.verify(EXPLOIT_ID)?;
        let buf = &mut proof.receipt.journal.bytes.as_slice();
        let vm_output = VmOutput::decode(buf);

        let provider = try_get_http_provider(self.rpc_url)?;

        for (block_number, hash) in vm_output.block_hashes.iter() {
            let block = provider.get_block(*block_number).await?.unwrap();
            if block.hash.unwrap().to_alloy() != *hash {
                bail!("block hash mismatch")
            }
        }
        serde_json::to_writer(self.output, &vm_output.state_diff)?;
        Ok(())
    }
}