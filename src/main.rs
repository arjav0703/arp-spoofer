use anyhow::Result;
use clap::Parser;
mod cli;
use pcap::Device;
use std::net::{IpAddr, Ipv4Addr};

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

    let network_interface = match get_network_interface(&interface) {
        Some(iface) => iface,
        None => {
            println!("Network interface '{}' not found.", interface);
            std::process::exit(1);
        }
    };
    // dbg!(&network_interface);

    let pcap_device = match Device::list()?
        .into_iter()
        .find(|d| d.name == network_interface.name)
    {
        Some(dev) => dev,
        None => {
            println!(
                "PCAP device for interface '{}' not found.",
                network_interface.name
            );
            std::process::exit(1);
        }
    };
    // dbg!(&pcap_device);

    let self_ip = pcap_device.ip_addr();
    let self_mac = network_interface.mac.unwrap();
    // dbg!(self_mac);

    Ok(())
}

fn get_network_interface(iface: &str) -> Option<pnet_datalink::NetworkInterface> {
    let interfaces = pnet_datalink::interfaces();
    for interface in interfaces {
        if interface.name == iface {
            return Some(interface);
        }
    }
    None
}

trait GetDetails {
    fn ip_addr(&self) -> Option<Ipv4Addr>;
}

impl GetDetails for pcap::Device {
    fn ip_addr(&self) -> Option<Ipv4Addr> {
        self.addresses
            .iter()
            .filter_map(|i| match i.addr {
                IpAddr::V4(ipv4) => Some(ipv4),
                _ => None,
            })
            .next_back()
    }
}

fn check_sudo() -> bool {
    use is_sudo::RunningAs;
    let is_sudo = is_sudo::check();

    match is_sudo {
        RunningAs::Root => true,
        RunningAs::User => false,
    }
}
