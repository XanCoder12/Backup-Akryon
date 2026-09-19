# Network Protocols (L2 to L4)

Nyxara OS includes native protocol implementations covering the Link Layer (Ethernet II), Address Resolution Protocol (ARP), Internet Protocol (IPv4), Internet Control Message Protocol (ICMP), User Datagram Protocol (UDP), and Transmission Control Protocol (TCP).

## 1. Ethernet II Framing (`ethernet.rs`)

Ethernet frames encapsulate all link-layer network traffic:

```
+-------------------+-------------------+-------------------+-------------------------+
| Dest MAC (6 B)    | Source MAC (6 B)  | EtherType (2 B)   | Payload (46 - 1500 B)   |
+-------------------+-------------------+-------------------+-------------------------+
```

- **Broadcast MAC**: `FF:FF:FF:FF:FF:FF`.
- **Supported EtherTypes**:
  - `0x0806`: Address Resolution Protocol (ARP).
  - `0x0800`: Internet Protocol Version 4 (IPv4).

## 2. Address Resolution Protocol (`arp.rs`)

ARP maps 32-bit IPv4 addresses to 48-bit Ethernet MAC addresses.

### ARP Header Layout
- **Hardware Type**: `0x0001` (Ethernet).
- **Protocol Type**: `0x0800` (IPv4).
- **Hardware & Protocol Length**: `6` (MAC) and `4` (IPv4).
- **Opcode**: `1` for ARP Request, `2` for ARP Reply.

### Dynamic ARP Cache
- Maintains a table of `(Ipv4Address, MacAddress, timestamp)` entries.
- `arp::resolve(ip, timeout_ms)`: Checks the cache first. If absent, broadcasts an ARP Request (`Who has X.X.X.X? Tell Y.Y.Y.Y`) and polls until an ARP Reply is received or the timeout expires.
- Inspectable in the shell using `arp -a` and flushable via `arp -c`.

## 3. Internet Protocol Version 4 (`ipv4.rs`)

IPv4 provides packet addressing, routing, and header checksum validation.

### Header Fields
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|Version|  IHL  |Type of Service|          Total Length         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         Identification        |Flags|      Fragment Offset    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Time to Live |    Protocol   |        Header Checksum        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       Source IPv4 Address                     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Destination IPv4 Address                   |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

- **Checksum**: Standard 16-bit one's complement sum of 16-bit words.
- **Subnet Routing (`route_destination`)**:
  - Compares `(target_ip & netmask) == (host_ip & netmask)`.
  - If target is on the local subnet: routes directly to `target_ip`.
  - If target is outside the subnet: routes to `gateway_ip`.

## 4. Internet Control Message Protocol (`icmp.rs`)

Provides network reachability diagnostics used by the `ping` utility:
- **Echo Request**: Type `8`, Code `0`. Contains Identifier, Sequence Number, and payload.
- **Echo Reply**: Type `0`, Code `0`. Matches Identifier and Sequence Number.
- **RTT Calculation**: Measures round-trip elapsed time in milliseconds via `timer_get_uptime_ms()`.

## 5. User Datagram Protocol (`udp.rs`)

A lightweight, connectionless transport protocol providing port multiplexing:
- **Header**: Source Port (16-bit), Destination Port (16-bit), Length (16-bit), Checksum (16-bit).
- **Pseudo-Header Checksum**: Calculates checksum over Source IP, Dest IP, Protocol (`17`), UDP Length, and UDP Header/Payload.
- Used by DHCP (Ports 67/68) and DNS (Port 53).

## 6. Transmission Control Protocol (`tcp.rs`)

Nyxara features an autonomous TCP engine implementing reliable, connection-oriented streaming.

### Flag Bitmask
- `TCP_FIN` (`0x01`): Connection teardown.
- `TCP_SYN` (`0x02`): Synchronize sequence numbers.
- `TCP_RST` (`0x04`): Connection reset.
- `TCP_PSH` (`0x08`): Push data to application.
- `TCP_ACK` (`0x10`): Acknowledgment field valid.

### State Machine Lifecycle
```
[Closed] ---> Send SYN ---> [SynSent]
                                |
                    Receive SYN-ACK, Send ACK
                                |
                                v
                        [Established]  <== Data Transfer
                                |
                 Send FIN / Receive FIN-ACK
                                |
                                v
                           [FinWait1]
                                |
                            [TimeWait] ---> [Closed]
```

- **3-Way Handshake**: Manages random Initial Sequence Numbers (ISN), negotiates connection parameters, and validates server ACKs.
- **Reliable Data Exchange**: Automatically tracks `seq` and `ack` counters, acknowledging inbound data segments.
- **Active and Passive Sockets**: Powers outbound client connections (`curl`) as well as inbound listening sockets (`httpd`).
