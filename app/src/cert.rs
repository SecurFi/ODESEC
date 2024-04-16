use clap::Parser;
use anyhow::Result;
use risc0_zkvm::{serde::to_vec, sha::Digest, ExecutorEnv, ExecutorImpl, Segment, SegmentRef};
use serde::{Deserialize, Serialize};
use x509_parser::pem::parse_x509_pem;
use guests::{CERT_ELF, CERT_ID};

#[derive(Parser, Debug)]
pub struct CertArgs {
    #[clap(short, long)]
    pub cert: String,

    /// Generate a SNARK ZK proof through Bonsai
    #[clap(short, long, default_value_t = false)]
    pub prove: bool,

    #[clap(short, long)]
    pub bonsai_key: Option<String>,
}

impl CertArgs {
    pub async fn run(self) -> Result<()> {
        println!("ImageId: {}", Digest::from(CERT_ID));
        let mut data = std::fs::read(self.cert)?;
        if matches!((data[0], data[1]), (0x30, 0x81..=0x83)) {

        } else {
            let (_, pem) = parse_x509_pem(&data)?;
            data = pem.contents
        }
        x509_parser::parse_x509_certificate(&data)?;
  
        println!("Running the executor...");
        let zkvm_input = to_vec(&data)?;
        let session = {
            let mut env_builder = ExecutorEnv::builder();
            env_builder.session_limit(None).write_slice(&zkvm_input);
            let env = env_builder.build().unwrap();
            let mut exec = ExecutorImpl::from_elf(env, CERT_ELF).unwrap();

            exec.run_with_callback(|_| Ok(Box::new(NULL_SEGMENT_REF)))
                .unwrap()
        };
        let journal = session.journal.unwrap();
        let domain: String = journal.decode()?;
        println!("domain: {}", domain);
        Ok(())
    }
}

const NULL_SEGMENT_REF: NullSegmentRef = NullSegmentRef {};
#[derive(Serialize, Deserialize)]
struct NullSegmentRef {}

impl SegmentRef for NullSegmentRef {
    fn resolve(&self) -> anyhow::Result<Segment> {
        unimplemented!()
    }
}