
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


fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Exploit(args) => args.run(),
        Commands::Cert(args) => args.run()
    }
}