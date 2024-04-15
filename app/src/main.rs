
use std::future::Future;
use cert::CertArgs;
use clap::{Parser, Subcommand};
use eyre::EyreHandler;

mod cert;
mod exploit;
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
    // Cert(CertArgs),
}


pub fn block_on<F: Future>(future: F) -> F::Output {
    let rt = tokio::runtime::Runtime::new().expect("could not start tokio rt");
    rt.block_on(future)
}


fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Exploit(args) => block_on(args.run()),
        // Commands::Cert(args) => args.run().await,
    }
}