# Network Stack Architecture

Nyxara OS implements a complete, zero-dependency TCP/IP network stack written in freestanding Rust (`rust/src/net/`). It operates directly on raw Ethernet frames exchanged with the Realtek RTL8139 Fast Ethernet driver.

## Layered Model

```
+-----------------------------------------------------------------+
|                       Application Layer                         |
|     DHCP Client | DNS Resolver | HTTP Client (curl) | httpd     |
+-----------------------------------------------------------------+
                                |
+-----------------------------------------------------------------+
|                       Transport Layer                           |
|       TCP (State Machine, 3-Way Handshake)    |     UDP         |
+-----------------------------------------------------------------+
                                |
+-----------------------------------------------------------------+
|                       Internet Layer                            |
|       IPv4 (Routing & Checksums)              |     ICMP        |
+-----------------------------------------------------------------+
                                |
+-----------------------------------------------------------------+
|                      Link / Driver Layer                        |
|       Ethernet II Framing                     |     ARP Cache   |
+-----------------------------------------------------------------+
                                |
+-----------------------------------------------------------------+
|            Hardware Layer (hal/rtl8139.c via FFI)               |
+-----------------------------------------------------------------+
```

## Hardware-to-Stack FFI Interface

The Rust networking subsystem interfaces with the C RTL8139 driver via explicit FFI bindings:

```rust
extern "C" {
    pub fn rtl8139_is_active() -> i32;
    pub fn rtl8139_get_mac(mac_out: *mut u8) -> i32;
    pub fn rtl8139_send_packet(data: *const u8, len: u32) -> i32;
    pub fn rtl8139_receive_packet(buf: *mut u8, max_len: u32) -> i32;
    pub fn rtl8139_get_stats(rx_pkts: *mut u32, tx_pkts: *mut u32,
                             rx_bytes: *mut u32, tx_bytes: *mut u32);
    pub fn timer_get_uptime_ms() -> u32;
}
```

## Network Configuration (`NetConfig`)

Interface settings for `eth0` are encapsulated in `NetConfig`:

```rust
pub struct NetConfig {
    pub mac: MacAddress,          // 6-byte Ethernet physical address
    pub ip: Ipv4Address,          // 4-byte IPv4 host address
    pub netmask: Ipv4Address,     // Subnet mask (e.g., 255.255.255.0)
    pub gateway: Ipv4Address,     // Default gateway (e.g., 10.0.2.2)
    pub dns: Ipv4Address,         // DNS nameserver (e.g., 10.0.2.3)
    pub is_up: bool,              // Interface link state
}
```

### Configuration Persistence in VFS
During boot, `net::init()` checks for `/etc/network.conf` in the VFS:
- If present, parameters (`IP`, `NETMASK`, `GATEWAY`, `DNS`) are parsed and applied.
- If absent, defaults matching standard QEMU User Networking (`10.0.2.15/24`, Gateway `10.0.2.2`, DNS `10.0.2.3`) are configured and written to `/etc/network.conf` and `/etc/resolv.conf`.

## Packet Polling and Dispatch Loop

Nyxara processes incoming packets via `net::poll_incoming()`:

1. **Hardware Ingestion**: `rtl8139_receive_packet()` pulls the raw frame from the ring buffer.
2. **Frame Parsing**: `ethernet::parse_frame()` validates minimum frame size (14 bytes) and extracts `dst_mac`, `src_mac`, and `ethertype`.
3. **Protocol Demultiplexing**:
   - `ETHERTYPE_ARP` (`0x0806`): Dispatches to `arp::handle_packet()`. Updates local ARP table cache and replies to ARP requests targeting `cfg.ip`.
   - `ETHERTYPE_IPV4` (`0x0800`): Dispatches to `ipv4::handle_packet()`. Verifies checksum, checks destination address, and routes based on IP protocol field (`1` for ICMP, `6` for TCP, `17` for UDP).
