
#[derive(Parser, Debug)]
pub struct CertArgs {
    #[clap(short, long)]
    pub cert: String,
}