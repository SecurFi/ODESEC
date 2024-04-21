use anyhow::Result;
use clap::Parser;
use guests::{CERT_ELF, CERT_ID};
use risc0_zkvm::sha::Digest;
use x509_parser::pem::parse_x509_pem;

use crate::prover::{execute, groth16_snark_encode, prove, stark2snark};

#[derive(Parser, Debug)]
pub struct CertArgs {
    #[clap(short, long)]
    pub cert: String,

    /// Generate a SNARK ZK proof through Bonsai, should set BONSAI_API_URL and BONSAI_API_KEY environment variables
    #[clap(short, long, default_value_t = false)]
    pub prove: bool,
}

impl CertArgs {
    pub fn run(self) -> Result<()> {
        println!("ImageId: {}", Digest::from(CERT_ID));
        let mut data: Vec<u8> = std::fs::read(self.cert)?;

        if matches!((data[0], data[1]), (0x30, 0x81..=0x83)) {
        } else {
            let (_, pem) = parse_x509_pem(&data)?;
            data = pem.contents
        }
        x509_parser::parse_x509_certificate(&data)?;

        let journal = execute(&data, 20, false, CERT_ELF, None);
        let domain = String::from_utf8(journal.bytes)?;
        println!("domain: {}", domain);
        if self.prove {
            let (stark_uuid, stark_receipt) = prove(&data, CERT_ELF, true, Default::default())?;

            let (_, snark_receipt) = stark2snark(stark_uuid, stark_receipt)?;

            let receipt_encoded = groth16_snark_encode(snark_receipt)?;
            println!("proof: {:?}", hex::encode(receipt_encoded));
        }
        Ok(())
    }
}
