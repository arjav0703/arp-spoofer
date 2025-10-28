# ARP Spoofer

**This tool is for educational and authorized security testing purposes only.**


## Features

- **MAC Address Discovery**: Automatically discovers target and gateway MAC addresses using ARP requests
- **Bidirectional Spoofing**: Poisons both target and gateway ARP caches simultaneously
- **IP Forwarding**: Automatically enables/disables IP forwarding to prevent DoS
- **Continuous Operation**: Sends spoofing packets every 2 seconds to maintain poisoned state
- **Graceful Shutdown**: Ctrl+C handler restores original ARP tables on exit
- **Cross-Platform**: Supports both macOS and Linux

## How It Works?

1. **Discovery Phase**: Sends ARP requests to discover the MAC addresses of both the target and gateway
2. **Enable Forwarding**: Enables IP forwarding so intercepted packets are forwarded instead of dropped
3. **Spoofing Loop**: Continuously sends poisoned ARP replies:
   - Tells target that attacker's MAC is the gateway
   - Tells gateway that attacker's MAC is the target
4. **Man-in-the-Middle**: All traffic between target and gateway flows through the attacker's machine
5. **Restoration**: On Ctrl+C, sends legitimate ARP packets to restore original MAC-IP associations

## Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Root/Admin Privileges**: Required for raw socket access and IP forwarding control
- **Network Interface**: Must know the name of your network interface (e.g., `en0` (macos), `eth0` (linux))

## Installation
```bash
cargo install arp-spoofer-cli
```

## Usage

```bash
sudo arp-spoofer-cli \
  --interface <INTERFACE> \
  --target <TARGET_IP> \
  --gateway <GATEWAY_IP>
```

### Parameters

- `-i, --interface <INTERFACE>`: Network interface to use (e.g., `en0`, `eth0`, `wlan0`)
- `-t, --target <TARGET_IP>`: IP address of the target device to spoof
- `-g, --gateway <GATEWAY_IP>`: IP address of the gateway/router

### Example
**This may vary on your machine**
```bash
# On macOS
sudo arp-spoofer-cli -i en0 -t 192.168.1.100 -g 192.168.1.1

# On Linux
sudo arp-spoofer-cli -i eth0 -t 192.168.1.100 -g 192.168.1.1
```

### Finding Your Interface

**macOS:**
```bash
ifconfig
# Look for en0, en1, etc.
```

**Linux:**
```bash
ip link show
# or
ifconfig
# Look for eth0, wlan0, etc.
```

### Finding Gateway IP

**macOS:**
```bash
netstat -nr | grep default
```

**Linux:**
```bash
ip route | grep default
# or
route -n | grep UG
```

## Technical Details

### Packet Structure

**Poisoned ARP Packet to Target:**
- Ethernet Dst: Target MAC
- Ethernet Src: Attacker MAC
- ARP Operation: Reply
- ARP Sender MAC: Attacker MAC (spoofing gateway)
- ARP Sender IP: Gateway IP
- ARP Target MAC: Target MAC
- ARP Target IP: Target IP

**Poisoned ARP Packet to Gateway:**
- Ethernet Dst: Gateway MAC
- Ethernet Src: Attacker MAC
- ARP Operation: Reply
- ARP Sender MAC: Attacker MAC (spoofing target)
- ARP Sender IP: Target IP
- ARP Target MAC: Gateway MAC
- ARP Target IP: Gateway IP

### IP Forwarding

The tool automatically enables IP forwarding to ensure intercepted packets are forwarded:

- **macOS**: Uses `sysctl -w net.inet.ip.forwarding=1`
- **Linux**: Writes `1` to `/proc/sys/net/ipv4/ip_forward`

On exit, it disables IP forwarding to restore the original state.

## Troubleshooting

### "This program must be run with sudo/root privileges"
Run the program with `sudo` or as root user.

### "Network interface 'xxx' not found"
Check available interfaces with `ifconfig` or `ip link show` and use the correct name.

### "Failed to enable IP forwarding"
Ensure you have root privileges. On some systems, IP forwarding may be restricted by security policies.

### No traffic being intercepted
- Verify both target and gateway are reachable
- Check that IP forwarding is enabled
- Some networks have ARP spoofing protection (Dynamic ARP Inspection)
- Firewalls may block forwarded traffic

### Target loses internet connectivity
If IP forwarding fails to enable, the target will lose connectivity. The tool will show a warning in this case.

