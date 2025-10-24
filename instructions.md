# ARP Spoofing - Step-by-Step Implementation Guide

## Overview
ARP spoofing (or ARP poisoning) tricks devices on a network into associating your MAC address with another device's IP address, allowing you to intercept traffic between two parties.

## Step-by-Step Implementation

### 1. **Discover Target MAC Address**
- Send an ARP request asking "Who has [target_ip]?"
- Listen for ARP reply containing the target's MAC address
- Store this MAC for later use
- Repeat for gateway IP to get gateway MAC

### 2. **Craft Poisoned ARP Packets**
You need to create two types of poisoned packets:

**Packet A - Sent to Target:**
- Source IP: Gateway IP
- Source MAC: Your MAC (not gateway's MAC - this is the spoof!)
- Destination MAC: Target's MAC
- Destination IP: Target IP
- ARP Operation: Reply (2)

**Packet B - Sent to Gateway:**
- Source IP: Target IP
- Source MAC: Your MAC (spoofing target)
- Destination MAC: Gateway's MAC
- Destination IP: Gateway IP
- ARP Operation: Reply (2)

### 3. **Enable IP Forwarding**
- Enable IP forwarding on your system so intercepted packets actually reach their destination
- On Linux/Mac: `echo 1 > /proc/sys/net/ipv4/ip_forward` or `sysctl -w net.inet.ip.forwarding=1`
- Without this, you'll DoS the connection instead of intercepting it

### 4. **Send Poisoned Packets Continuously**
- Send both packets (to target and gateway) in a loop
- Send every 1-3 seconds to maintain the poisoned state
- ARP caches expire/refresh, so continuous sending is necessary
- Handle Ctrl+C gracefully to restore original ARP tables

### 5. **Packet Sending Details**
- Use raw sockets or a library like `pnet` to craft Ethernet frames
- Set Ethernet type to ARP (0x0806)
- Build proper ARP packet structure:
  - Hardware type: Ethernet (1)
  - Protocol type: IPv4 (0x0800)
  - Hardware size: 6 bytes
  - Protocol size: 4 bytes
  - Opcode: Reply (2)

### 6. **Restoration on Exit**
When program terminates:
- Send legitimate ARP packets to restore original MAC-IP associations
- Send target's real MAC to gateway
- Send gateway's real MAC to target
- Send multiple times (3-5) to ensure cache update
- Re-disable IP forwarding if you enabled it

### 7. **Optional Enhancements**
- **Verbose mode**: Show sent packets and traffic stats
- **Packet sniffing**: Capture and display intercepted traffic
- **Protocol filtering**: Only show HTTP, DNS, etc.
- **SSL stripping**: Downgrade HTTPS to HTTP (advanced, use existing tools)
- **Traffic modification**: Alter packets in transit

## Key Rust Crates You'll Need
- `pnet` or `pcap` - for packet crafting and sending
- `pnet_datalink` - for layer 2 operations
- `ctrlc` - for handling graceful shutdown
- `tokio` or `async-std` - for asynchronous packet sending

## Security Considerations
- **Only use on networks you own or have explicit permission to test**
- ARP spoofing is illegal on networks without authorization
- This is a network attack that can disrupt services
- Modern networks may have ARP spoofing protection (DAI, port security)

## Architecture Suggestion
```
main()
  ├── Discover target & gateway MACs
  ├── Enable IP forwarding
  ├── Setup Ctrl+C handler
  ├── Start spoofing loop
  │   ├── Send poisoned ARP to target
  │   ├── Send poisoned ARP to gateway
  │   └── Sleep (1-3 seconds)
  └── On exit: restore ARPs & disable forwarding
```

Your current code already has the interface selection and MAC/IP discovery foundation. Next, you'll need to implement ARP discovery and the poisoning loop.
