#![allow(dead_code)]
use std::io::{Read, Write};
// use chains_evm::{deal::DealRecord, evm_primitives::U256};
use anyhow::Result;
use risc0_zkvm::Receipt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Proof {
    pub version: String,
    pub image_id: [u32; 8],
    pub chain: String,
    pub receipt: Receipt,
}



impl Proof {
    pub fn load<R: Read>(input: R) -> Result<Self> {
        let data = bincode::deserialize_from(input)?;
        Ok(data)
    }

    pub fn save<W: Write>(&self, output: W) -> Result<()> {
        bincode::serialize_into(output, self)?;
        Ok(())
    }
}