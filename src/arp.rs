use super::*;
use pnet::datalink::MacAddr;
use pnet::datalink::NetworkInterface;
use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::{MutablePacket, Packet};
use std::net::Ipv4Addr;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ArpSpoofer {
    pub target_ip: IpAddr,
    pub gateway_ip: IpAddr,
    pub target_mac: [u8; 6],
    pub gateway_mac: [u8; 6],
    pub self_ip: Ipv4Addr,
    pub self_mac: [u8; 6],
    pub interface: NetworkInterface,
}

impl ArpSpoofer {
    pub fn find_target_mac(&self) -> [u8; 6] {
        self.get_mac_from_ip(self.target_ip, &self.interface)
            .unwrap()
    }

    pub fn find_gateway_mac(&self) -> [u8; 6] {
        self.get_mac_from_ip(self.gateway_ip, &self.interface)
            .unwrap()
    }

    pub fn get_mac_from_ip(
        &self,
        target_ip: IpAddr,
        interface: &NetworkInterface,
    ) -> Option<[u8; 6]> {
        let source_mac = array_to_mac(self.self_mac);
        let source_ip = self.self_ip;

        // Create a datalink channel
        let (mut tx, mut rx) = match datalink::channel(interface, Default::default()) {
            Ok(Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => panic!("Unhandled channel type"),
            Err(e) => panic!("Failed to create datalink channel: {}", e),
        };

        // Build ARP request
        let mut ethernet_buffer = [0u8; 42];
        {
            let mut eth_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();
            eth_packet.set_destination(MacAddr::broadcast());
            eth_packet.set_source(source_mac);
            eth_packet.set_ethertype(EtherTypes::Arp);

            let mut arp_packet = MutableArpPacket::new(eth_packet.payload_mut()).unwrap();
            arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
            arp_packet.set_protocol_type(EtherTypes::Ipv4);
            arp_packet.set_hw_addr_len(6);
            arp_packet.set_proto_addr_len(4);
            arp_packet.set_operation(ArpOperations::Request);
            arp_packet.set_sender_hw_addr(source_mac);
            arp_packet.set_sender_proto_addr(source_ip);
            arp_packet.set_target_hw_addr(MacAddr::zero());
            arp_packet.set_target_proto_addr(match target_ip {
                IpAddr::V4(ipv4) => ipv4,
                _ => panic!("Target IP is not IPv4"),
            });
        }

        // Send the packet
        tx.send_to(&ethernet_buffer, Some(interface.clone()))
            .expect("Failed to send ARP request")
            .unwrap();

        // Wait for response
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(3) {
            match rx.next() {
                Ok(packet) => {
                    let eth_packet = EthernetPacket::new(packet).unwrap();
                    if eth_packet.get_ethertype() == EtherTypes::Arp
                        && let Some(arp_packet) = ArpPacket::new(eth_packet.payload())
                        && arp_packet.get_operation() == ArpOperations::Reply
                        && arp_packet.get_sender_proto_addr() == target_ip
                    {
                        let mac = arp_packet.get_sender_hw_addr();
                        return Some(mac_to_array(mac));
                    }
                }
                Err(_) => continue,
            }

            thread::sleep(Duration::from_millis(10));
        }

        None
    }
}

fn mac_to_array(mac: pnet::datalink::MacAddr) -> [u8; 6] {
    [mac.0, mac.1, mac.2, mac.3, mac.4, mac.5]
}

fn array_to_mac(arr: [u8; 6]) -> pnet::datalink::MacAddr {
    pnet::datalink::MacAddr::new(arr[0], arr[1], arr[2], arr[3], arr[4], arr[5])
}
