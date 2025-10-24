use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub debug: bool,

    #[arg(long, short)]
    pub interface: String,

    #[arg(long, short)]
    pub target: String,

    #[arg(long, short)]
    pub gateway: String,
}
