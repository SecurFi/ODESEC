
use std::future::Future;
use cert::CertArgs;
use clap::{Parser, Subcommand};
use anyhow::Result;
mod cert;
mod exploit;
mod prover;
use exploit::ExploitArgs;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Exploit(ExploitArgs),
    Cert(CertArgs),
}


pub fn block_on<F: Future>(future: F) -> F::Output {
    let rt = tokio::runtime::Runtime::new().expect("could not start tokio rt");
    rt.block_on(future)
}


fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Exploit(args) => block_on(args.run()),
        Commands::Cert(args) => block_on(args.run()),
    }
}