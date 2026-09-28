use alloc::vec::Vec;
use core::cell::UnsafeCell;
use super::ipv4::{self, Ipv4Address, Ipv4Packet, PROTO_TCP};

extern "C" {
    fn timer_get_uptime_ms() -> u32;
}

pub const TCP_FIN: u16 = 0x0001;
pub const TCP_SYN: u16 = 0x0002;
pub const TCP_RST: u16 = 0x0004;
pub const TCP_PSH: u16 = 0x0008;
pub const TCP_ACK: u16 = 0x0010;

const DEFAULT_MSS: usize = 536;
const LOCAL_MSS: u16 = 1460;
const RTO_BASE_MS: u32 = 1000;
const MAX_RETRIES: u8 = 5;
const MAX_SOCKETS: usize = 16;
const MAX_RETRANS_QUEUE: usize = 32;
const TIME_WAIT_MS: u32 = 2000;
const RX_BUF_LIMIT: usize = 65536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    LastAck,
    TimeWait,
}

#[derive(Debug, Clone)]
pub struct TcpSegment {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq: u32,
    pub ack: u32,
    pub flags: u16,
    pub window: u16,
    pub mss: Option<u16>,
    pub payload: Vec<u8>,
}

impl TcpSegment {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 20 {
            return None;
        }

        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let seq = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ack = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        let data_offset = ((data[12] >> 4) as usize) * 4;
        let flags = u16::from_be_bytes([data[12] & 0x01, data[13]]);
        let window = u16::from_be_bytes([data[14], data[15]]);

        if data_offset < 20 || data.len() < data_offset {
            return None;
        }

        let mut mss = None;
        let mut i = 20;
        while i < data_offset {
            match data[i] {
                0 | 1 => i += 1,
                kind => {
                    if i + 1 >= data_offset {
                        break;
                    }
                    let len = data[i + 1] as usize;
                    if len < 2 || i + len > data_offset {
                        break;
                    }
                    if kind == 2 && len == 4 {
                        mss = Some(u16::from_be_bytes([data[i + 2], data[i + 3]]));
                    }
                    i += len;
                }
            }
        }

        let payload = data[data_offset..].to_vec();

        Some(Self {
            src_port,
            dst_port,
            seq,
            ack,
            flags,
            window,
            mss,
            payload,
        })
    }

    pub fn to_bytes(&self, src_ip: Ipv4Address, dst_ip: Ipv4Address) -> Vec<u8> {
        let header_len = if self.mss.is_some() { 24 } else { 20 };
        let total_len = (header_len + self.payload.len()) as u16;
        let mut pseudo = Vec::with_capacity(12 + total_len as usize);

        pseudo.extend_from_slice(&src_ip);
        pseudo.extend_from_slice(&dst_ip);
        pseudo.push(0);
        pseudo.push(PROTO_TCP);
        pseudo.extend_from_slice(&total_len.to_be_bytes());

        let mut tcp_hdr = Vec::with_capacity(total_len as usize);
        tcp_hdr.extend_from_slice(&self.src_port.to_be_bytes());
        tcp_hdr.extend_from_slice(&self.dst_port.to_be_bytes());
        tcp_hdr.extend_from_slice(&self.seq.to_be_bytes());
        tcp_hdr.extend_from_slice(&self.ack.to_be_bytes());

        let doff_flags = ((header_len as u16 / 4) << 12) | (self.flags & 0x01FF);
        tcp_hdr.extend_from_slice(&doff_flags.to_be_bytes());
        tcp_hdr.extend_from_slice(&self.window.to_be_bytes());
        tcp_hdr.extend_from_slice(&[0, 0]); // Checksum placeholder
        tcp_hdr.extend_from_slice(&[0, 0]); // Urgent pointer
        if let Some(m) = self.mss {
            tcp_hdr.extend_from_slice(&[2, 4]);
            tcp_hdr.extend_from_slice(&m.to_be_bytes());
        }
        tcp_hdr.extend_from_slice(&self.payload);

        pseudo.extend_from_slice(&tcp_hdr);
        let csum = ipv4::checksum(&pseudo);
        tcp_hdr[16..18].copy_from_slice(&csum.to_be_bytes());
        tcp_hdr
    }
}

pub struct TcpSocket {
    pub local_port: u16,
    pub remote_ip: Ipv4Address,
    pub remote_port: u16,
    pub state: TcpState,
    pub local_seq: u32,
    pub remote_seq: u32,
    pub rx_buf: Vec<u8>,
    pub is_listener: bool,
    pub connected: bool,
    pub peer_closed: bool,
    pub reset: bool,
    pub mss: usize,
    pub retrans: Vec<UnackedSeg>,
    pub free: bool,
    pub gen: u32,
    pub state_ms: u32,
}

#[derive(Debug, Clone)]
pub struct UnackedSeg {
    seq: u32,
    flags: u16,
    data: Vec<u8>,
    sent_ms: u32,
    retries: u8,
}

fn now_ms() -> u32 {
    unsafe { timer_get_uptime_ms() }
}

fn seq_before(a: u32, b: u32) -> bool {
    ((a.wrapping_sub(b)) as i32) < 0
}

fn synfin_len(flags: u16) -> u32 {
    if (flags & (TCP_SYN | TCP_FIN)) != 0 {
        1
    } else {
        0
    }
}

fn isn(port: u16) -> u32 {
    now_ms().wrapping_mul(1_103_515_245).wrapping_add((port as u32).wrapping_mul(40_503))
}

struct SafeTcpManager(UnsafeCell<Vec<TcpSocket>>);
unsafe impl Sync for SafeTcpManager {}

static TCP_MANAGER: SafeTcpManager = SafeTcpManager(UnsafeCell::new(Vec::new()));

fn sockets() -> &'static mut Vec<TcpSocket> {
    unsafe { &mut *TCP_MANAGER.0.get() }
}

fn bind_slot(sockets: &mut Vec<TcpSocket>, sock: TcpSocket) -> Option<(usize, u32)> {
    if let Some((i, slot)) = sockets.iter_mut().enumerate().find(|(_, s)| s.free) {
        let gen = slot.gen + 1;
        *slot = sock;
        return Some((i, gen));
    }
    if sockets.len() < MAX_SOCKETS {
        sockets.push(sock);
        return Some((sockets.len() - 1, 1));
    }
    None
}

fn send_segment(sock: &TcpSocket, flags: u16, payload: &[u8]) -> Result<(), &'static str> {
    let cfg = match super::get_config() {
        Some(c) => c,
        None => return Err("Network uninitialized"),
    };

    let seg = TcpSegment {
        src_port: sock.local_port,
        dst_port: sock.remote_port,
        seq: sock.local_seq,
        ack: sock.remote_seq,
        flags,
        window: 8192,
        mss: if (flags & TCP_SYN) != 0 { Some(LOCAL_MSS) } else { None },
        payload: payload.to_vec(),
    };

    let bytes = seg.to_bytes(cfg.ip, sock.remote_ip);
    super::ipv4_send(sock.remote_ip, PROTO_TCP, bytes)
}

fn transmit(sock: &mut TcpSocket, flags: u16, payload: &[u8]) -> Result<(), &'static str> {
    let seq = sock.local_seq;
    let res = send_segment(sock, flags, payload);
    if (flags & TCP_SYN) != 0 || !payload.is_empty() {
        if sock.retrans.len() >= MAX_RETRANS_QUEUE {
            sock.retrans.remove(0);
        }
        sock.retrans.push(UnackedSeg {
            seq,
            flags,
            data: payload.to_vec(),
            sent_ms: now_ms(),
            retries: 0,
        });
    }
    res
}

fn send_data(sock: &mut TcpSocket, data: &[u8]) -> Result<(), &'static str> {
    if sock.state != TcpState::Established {
        return Err("Socket not in ESTABLISHED state");
    }
    let mss = sock.mss.max(1).min(LOCAL_MSS as usize);
    for chunk in data.chunks(mss) {
        if sock.retrans.len() >= MAX_RETRANS_QUEUE {
            return Err("TCP retransmit queue full");
        }
        let _ = transmit(sock, TCP_PSH | TCP_ACK, chunk);
        sock.local_seq = sock.local_seq.wrapping_add(chunk.len() as u32);
    }
    Ok(())
}

fn send_rst(ip_pkt: &Ipv4Packet, seg: &TcpSegment) {
    let ack = seg
        .seq
        .wrapping_add(seg.payload.len() as u32)
        .wrapping_add(synfin_len(seg.flags));
    let rst = TcpSegment {
        src_port: seg.dst_port,
        dst_port: seg.src_port,
        seq: 0,
        ack,
        flags: TCP_RST | TCP_ACK,
        window: 0,
        mss: None,
        payload: Vec::new(),
    };
    let bytes = rst.to_bytes(ip_pkt.dst, ip_pkt.src);
    let _ = super::ipv4_send(ip_pkt.src, PROTO_TCP, bytes);
}

fn process_segment(sock: &mut TcpSocket, seg: &TcpSegment) {
    if (seg.flags & TCP_RST) != 0 {
        sock.reset = true;
        sock.connected = false;
        sock.state = TcpState::Closed;
        sock.state_ms = now_ms();
        sock.retrans.clear();
        return;
    }

    if (seg.flags & TCP_ACK) != 0 {
        sock.retrans.retain(|e| {
            let end = e
                .seq
                .wrapping_add(e.data.len() as u32)
                .wrapping_add(synfin_len(e.flags));
            seq_before(seg.ack, end)
        });
    }

    let seg_end = seg.seq.wrapping_add(seg.payload.len() as u32);

    match sock.state {
        TcpState::SynSent => {
            if (seg.flags & TCP_SYN) != 0 {
                sock.mss = seg
                    .mss
                    .map(|m| m as usize)
                    .unwrap_or(DEFAULT_MSS)
                    .clamp(1, LOCAL_MSS as usize);
                if sock.mss == 0 {
                    sock.mss = DEFAULT_MSS;
                }
                sock.remote_seq = seg.seq.wrapping_add(1);
                sock.local_seq = seg.ack;
                sock.state = TcpState::Established;
                sock.state_ms = now_ms();
                sock.connected = true;
                let _ = send_segment(sock, TCP_ACK, &[]);
            }
        }
        TcpState::SynReceived => {
            if (seg.flags & TCP_ACK) != 0 {
                sock.state = TcpState::Established;
                sock.state_ms = now_ms();
                sock.connected = true;
            }
        }
        TcpState::Established => {
            if !seg.payload.is_empty() {
                if seg.seq == sock.remote_seq && sock.rx_buf.len() < RX_BUF_LIMIT {
                    sock.rx_buf.extend_from_slice(&seg.payload);
                    sock.remote_seq = seg_end;
                }
                let _ = send_segment(sock, TCP_ACK, &[]);
            }
            if (seg.flags & TCP_FIN) != 0 && seg_end == sock.remote_seq {
                sock.remote_seq = seg_end.wrapping_add(1);
                sock.peer_closed = true;
                sock.state = TcpState::CloseWait;
                sock.state_ms = now_ms();
                let _ = send_segment(sock, TCP_ACK, &[]);
            }
        }
        TcpState::FinWait1 => {
            if (seg.flags & TCP_ACK) != 0 && seg.ack == sock.local_seq {
                sock.state = TcpState::FinWait2;
                sock.state_ms = now_ms();
            }
            if (seg.flags & TCP_FIN) != 0 && seg_end == sock.remote_seq {
                sock.remote_seq = seg_end.wrapping_add(1);
                let _ = send_segment(sock, TCP_ACK, &[]);
                sock.state = TcpState::TimeWait;
                sock.state_ms = now_ms();
            }
        }
        TcpState::FinWait2 => {
            if (seg.flags & TCP_FIN) != 0 && seg_end == sock.remote_seq {
                sock.remote_seq = seg_end.wrapping_add(1);
                let _ = send_segment(sock, TCP_ACK, &[]);
                sock.state = TcpState::TimeWait;
                sock.state_ms = now_ms();
            }
        }
        TcpState::LastAck => {
            if (seg.flags & TCP_ACK) != 0 && seg.ack == sock.local_seq {
                sock.state = TcpState::Closed;
                sock.state_ms = now_ms();
            }
        }
        TcpState::CloseWait | TcpState::TimeWait => {
            if (seg.flags & TCP_FIN) != 0 {
                let _ = send_segment(sock, TCP_ACK, &[]);
            }
        }
        _ => {}
    }
}

pub fn handle_packet(ip_pkt: &Ipv4Packet) {
    let seg = match TcpSegment::parse(&ip_pkt.payload) {
        Some(s) => s,
        None => return,
    };

    let sockets = sockets();

    for sock in sockets.iter_mut() {
        if sock.free || sock.is_listener {
            continue;
        }
        if sock.local_port == seg.dst_port
            && sock.remote_ip == ip_pkt.src
            && sock.remote_port == seg.src_port
        {
            process_segment(sock, &seg);
            return;
        }
    }

    if (seg.flags & TCP_SYN) != 0 && (seg.flags & TCP_ACK) == 0 {
        if let Some(lport) = sockets
            .iter()
            .find(|s| !s.free && s.is_listener && s.local_port == seg.dst_port)
            .map(|s| s.local_port)
        {
            let mss = seg
                .mss
                .map(|m| m as usize)
                .unwrap_or(DEFAULT_MSS)
                .clamp(1, LOCAL_MSS as usize);
            let child = TcpSocket {
                local_port: lport,
                remote_ip: ip_pkt.src,
                remote_port: seg.src_port,
                state: TcpState::SynReceived,
                local_seq: isn(lport),
                remote_seq: seg.seq.wrapping_add(1),
                rx_buf: Vec::new(),
                is_listener: false,
                connected: false,
                peer_closed: false,
                reset: false,
                mss,
                retrans: Vec::new(),
                free: false,
                gen: 0,
                state_ms: now_ms(),
            };

            match bind_slot(sockets, child) {
                Some((id, _gen)) => {
                    let sock = &mut sockets[id];
                    let _ = transmit(sock, TCP_SYN | TCP_ACK, &[]);
                    sock.local_seq = sock.local_seq.wrapping_add(1);
                }
                None => return,
            }
            return;
        }
    }

    if (seg.flags & TCP_RST) == 0 {
        send_rst(ip_pkt, &seg);
    }
}

pub struct TcpStream {
    pub socket_id: usize,
    gen: u32,
}

static mut NEXT_CLIENT_PORT: u16 = 49152;

pub fn connect(remote_ip: Ipv4Address, remote_port: u16, timeout_ms: u32) -> Result<TcpStream, &'static str> {
    let local_port = unsafe {
        let p = NEXT_CLIENT_PORT;
        NEXT_CLIENT_PORT = if p >= 65000 { 49152 } else { p + 1 };
        p
    };

    let sockets = sockets();
    let sock = TcpSocket {
        local_port,
        remote_ip,
        remote_port,
        state: TcpState::SynSent,
        local_seq: isn(local_port),
        remote_seq: 0,
        rx_buf: Vec::new(),
        is_listener: false,
        connected: false,
        peer_closed: false,
        reset: false,
        mss: DEFAULT_MSS,
        retrans: Vec::new(),
        free: false,
        gen: 0,
        state_ms: now_ms(),
    };

    let (socket_id, gen) = match bind_slot(sockets, sock) {
        Some(v) => v,
        None => return Err("Too many open TCP sockets"),
    };

    let s = &mut sockets[socket_id];
    let _ = transmit(s, TCP_SYN, &[]);
    s.local_seq = s.local_seq.wrapping_add(1);

    let start = now_ms();
    while now_ms().wrapping_sub(start) < timeout_ms {
        super::poll();
        let s = &sockets[socket_id];
        if s.connected && s.gen == gen {
            return Ok(TcpStream { socket_id, gen });
        }
        if s.reset {
            return Err("Connection refused");
        }
    }

    let s = &mut sockets[socket_id];
    if s.gen == gen && !s.connected {
        s.free = true;
        s.retrans.clear();
    }
    Err("TCP connection timed out (handshake failed)")
}

impl TcpStream {
    fn socket(&mut self) -> Result<&mut TcpSocket, &'static str> {
        let sockets = sockets();
        match sockets.get_mut(self.socket_id) {
            Some(s) if !s.free && s.gen == self.gen => Ok(s),
            _ => Err("Invalid socket ID"),
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), &'static str> {
        let sock = self.socket()?;
        if sock.reset {
            return Err("Connection reset by peer");
        }
        send_data(sock, data)
    }

    pub fn read(&mut self, timeout_ms: u32) -> Result<Vec<u8>, &'static str> {
        let start = now_ms();
        loop {
            super::poll();
            let sock = self.socket()?;
            if sock.reset {
                return Err("Connection reset by peer");
            }
            if !sock.rx_buf.is_empty() {
                return Ok(core::mem::replace(&mut sock.rx_buf, Vec::new()));
            }
            if sock.peer_closed || sock.state == TcpState::Closed || sock.state == TcpState::CloseWait {
                return Ok(Vec::new());
            }
            if now_ms().wrapping_sub(start) >= timeout_ms {
                break;
            }
        }
        Ok(Vec::new())
    }

    pub fn close(&mut self) {
        let sock = match self.socket() {
            Ok(s) => s,
            Err(_) => return,
        };
        match sock.state {
            TcpState::Established | TcpState::SynReceived | TcpState::SynSent => {
                let _ = transmit(sock, TCP_FIN | TCP_ACK, &[]);
                sock.local_seq = sock.local_seq.wrapping_add(1);
                sock.state = TcpState::FinWait1;
            }
            TcpState::CloseWait => {
                let _ = transmit(sock, TCP_FIN | TCP_ACK, &[]);
                sock.local_seq = sock.local_seq.wrapping_add(1);
                sock.state = TcpState::LastAck;
            }
            _ => {}
        }
        sock.state_ms = now_ms();
    }
}

pub fn listen(port: u16) -> Result<usize, &'static str> {
    let sockets = sockets();
    for (idx, sock) in sockets.iter().enumerate() {
        if !sock.free && sock.is_listener && sock.local_port == port {
            return Ok(idx);
        }
    }

    let sock = TcpSocket {
        local_port: port,
        remote_ip: [0, 0, 0, 0],
        remote_port: 0,
        state: TcpState::Listen,
        local_seq: 0,
        remote_seq: 0,
        rx_buf: Vec::new(),
        is_listener: true,
        connected: false,
        peer_closed: false,
        reset: false,
        mss: DEFAULT_MSS,
        retrans: Vec::new(),
        free: false,
        gen: 0,
        state_ms: now_ms(),
    };

    match bind_slot(sockets, sock) {
        Some((idx, _)) => Ok(idx),
        None => Err("Socket table full"),
    }
}

pub fn get_socket_count() -> usize {
    sockets().iter().filter(|s| !s.free).count()
}

pub fn service_http_server(port: u16) -> bool {
    let sockets = sockets();
    for sock in sockets.iter_mut() {
        if sock.free || sock.is_listener {
            continue;
        }
        if sock.local_port == port && sock.state == TcpState::Established && !sock.rx_buf.is_empty() {
            if let Ok(req_str) = core::str::from_utf8(&sock.rx_buf) {
                let resp = super::http::handle_http_request(req_str);
                sock.rx_buf.clear();
                if send_data(sock, resp.as_bytes()).is_err() {
                    return true;
                }
                let _ = transmit(sock, TCP_FIN | TCP_ACK, &[]);
                sock.local_seq = sock.local_seq.wrapping_add(1);
                sock.state = TcpState::FinWait1;
                sock.state_ms = now_ms();
                return true;
            }
        }
    }
    false
}

pub fn close_listener(port: u16) {
    let sockets = sockets();
    for sock in sockets.iter_mut() {
        if sock.is_listener && sock.local_port == port {
            sock.free = true;
        }
    }
}

pub fn retransmit_tick() {
    let now = now_ms();
    let sockets = sockets();
    for sock in sockets.iter_mut() {
        if sock.free {
            continue;
        }

        if (sock.state == TcpState::TimeWait || sock.state == TcpState::Closed)
            && now.wrapping_sub(sock.state_ms) > TIME_WAIT_MS
        {
            sock.free = true;
            sock.retrans.clear();
            continue;
        }

        let mut i = 0;
        while i < sock.retrans.len() {
            let e = &mut sock.retrans[i];
            let rto = RTO_BASE_MS << e.retries.min(3);
            if now.wrapping_sub(e.sent_ms) < rto {
                i += 1;
                continue;
            }
            if e.retries >= MAX_RETRIES {
                sock.reset = true;
                sock.connected = false;
                sock.state = TcpState::Closed;
                sock.state_ms = now;
                sock.retrans.clear();
                break;
            }
            e.retries += 1;
            e.sent_ms = now;
            let seg = TcpSegment {
                src_port: sock.local_port,
                dst_port: sock.remote_port,
                seq: e.seq,
                ack: sock.remote_seq,
                flags: e.flags,
                window: 8192,
                mss: None,
                payload: e.data.clone(),
            };
            if let Some(cfg) = super::get_config() {
                let bytes = seg.to_bytes(cfg.ip, sock.remote_ip);
                let _ = super::ipv4_send(sock.remote_ip, PROTO_TCP, bytes);
            }
            i += 1;
        }
    }
}
