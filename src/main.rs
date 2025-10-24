use anyhow::Result;
use clap::Parser;
mod cli;
use std::net::IpAddr;

fn main() -> Result<()> {
    if !check_sudo() {
        println!("This program must be run with sudo/root privileges.");
        std::process::exit(1);
    }

    let cli = cli::Cli::parse();
    let (debug, interface, target_ip, gateway_ip) =
        (cli.debug, cli.interface, cli.target, cli.gateway);

    let target_ip: IpAddr = target_ip.parse()?;
    let gateway_ip: IpAddr = gateway_ip.parse()?;

    Ok(())
}

fn check_sudo() -> bool {
    use is_sudo::RunningAs;
    let is_sudo = is_sudo::check();

    match is_sudo {
        RunningAs::Root => true,
        RunningAs::User => false,
    }
}
