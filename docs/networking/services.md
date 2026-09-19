# Network Services & Application Layer

Nyxara OS provides a suite of user-facing network services and diagnostics, including dynamic IP leasing (DHCP), domain name resolution (DNS), an HTTP client (`curl`), an embedded web server (`httpd`), and arbitrary payload transmission via Netcat (`nc`).

## 1. Dynamic Host Configuration Protocol (DHCP)

The DHCP client (`rust/src/net/dhcp.rs`) implements the 4-step DORA lifecycle over UDP ports 67 (Server) and 68 (Client).

### The DORA Lifecycle
1. **Discover**: Client broadcasts a DHCP Discover packet to `255.255.255.255:67` with its MAC address and transaction ID (`xid`).
2. **Offer**: DHCP Server replies with a proposed IP address (`yiaddr`), server identifier, and gateway options.
3. **Request**: Client broadcasts a DHCP Request formally requesting the offered IP.
4. **Acknowledge (ACK)**: Server confirms the lease duration and network configuration.

### Supported DHCP Options
- Magic Cookie: `99, 130, 83, 99` (`0x63825363`).
- **Option 1**: Subnet Mask.
- **Option 3**: Default Router / Gateway.
- **Option 6**: Domain Name Server (DNS).
- **Option 51**: IP Address Lease Time (seconds).
- **Option 53**: DHCP Message Type (1=Discover, 2=Offer, 3=Request, 5=Ack).

Invoked via shell command: `dhcp`.

## 2. Domain Name System (DNS) Resolver

The DNS client (`rust/src/net/dns.rs`) resolves human-readable domain names into 32-bit IPv4 addresses over UDP port 53.

### Query Construction
1. Generates a 12-byte DNS Header with Recursion Desired (`RD = 1`).
2. Encodes the query name as length-prefixed labels (e.g. `example.com` $\rightarrow$ `\x07example\x03com\x00`).
3. Appends Query Type `A` (`0x0001`) and Query Class `IN` (`0x0001`).
4. Sends the datagram to the configured nameserver (from `/etc/resolv.conf`).

### Response Parsing
- Parses answer resource records, handling DNS pointer compression (`0xC0` prefix).
- Extracts and returns the first valid Type A 4-byte IPv4 address.
- Invoked via shell commands: `dns <hostname>` or `nslookup <hostname>`.

## 3. HTTP Client (`curl` / `fetch`)

Implemented in `rust/src/net/http.rs`:
- Parses URL schema (`http://<host>[:port]/path`).
- Automatically invokes the DNS resolver if the host is a domain name.
- Establishes a TCP connection to the target port (default 80).
- Transmits an RFC-compliant HTTP/1.1 request:
  ```http
  GET /path HTTP/1.1
  Host: example.com
  User-Agent: Nyxara/2.0 (x86; no_std)
  Connection: close
  Accept: */*
  ```
- Collects response segments, splits status line, headers, and body, and prints output to the console.
- Invoked via shell command: `curl <url>`.

## 4. Embedded Web Server (`httpd`)

Nyxara includes a built-in HTTP server listening on port 80 (or a user-specified port):
- Accepts inbound TCP connection requests from external web browsers or tools.
- Parses incoming HTTP `GET` requests.
- Dynamically renders a system dashboard response containing:
  - System uptime and current RTC timestamp.
  - Physical memory utilization (PMM free vs used frames).
  - Kernel heap statistics.
  - Network interface packet counters.
- Transmits HTTP headers (`HTTP/1.1 200 OK`, `Content-Type: text/html`) followed by HTML, and initiates graceful TCP teardown.
- Invoked via shell command: `httpd [port]`.

## 5. Raw Network Utility (`nc` / Netcat)

Permits arbitrary socket transmissions:
- `nc <ip> <port>`: Establishes a TCP connection, transmits text payloads, and prints server replies.
- `nc -u <ip> <port>`: Transmits raw UDP datagrams to target hosts.
