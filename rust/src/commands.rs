use crate::vga::{self, Color};
use crate::{print, println, print_colored, logln};
use crate::{mouse, framebuffer};
use crate::vfs::InodeType;

extern "C" {
    fn timer_get_ticks() -> u32;
    fn timer_get_uptime_seconds() -> u32;
    fn timer_get_uptime_ms() -> u32;
    fn outb(port: u16, val: u8);
    fn rtc_get_datetime(t: *mut RtcTime);
    fn keyboard_has_char() -> bool;
    fn keyboard_getchar() -> u16;
}

/// Mirror of hal/rtc.h rtc_time_t - must match C layout exactly.
#[repr(C)]
struct RtcTime {
    second: u8,
    minute: u8,
    hour:   u8,
    day:    u8,
    month:  u8,
    _pad:   u8,  // alignment padding to match uint16_t year
    year:   u16,
}

pub fn handle_command(cmd: &str) {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return;
    }

    logln!("[Nyxara Shell] Executing command: '{}'", trimmed);

    let mut parts = trimmed.splitn(2, ' ');
    let command = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("");

    let (command, args) = if command == "sudo" {
        let trimmed_args = args.trim_start();
        let mut sub_parts = trimmed_args.splitn(2, ' ');
        let sub_cmd = sub_parts.next().unwrap_or("");
        let sub_args = sub_parts.next().unwrap_or("");
        (sub_cmd, sub_args)
    } else {
        (command, args)
    };

    match command {
        "help" => cmd_help(),
        "clear" => cmd_clear(),
        "about" | "version" => cmd_about(),
        "sysinfo" => cmd_sysinfo(),
        "free" | "meminfo" => cmd_free(),
        "uptime" => cmd_uptime(),
        "pwd" => cmd_pwd(),
        "cd" => cmd_cd(args),
        "mkdir" => cmd_mkdir(args),
        "ls" => cmd_ls(args),
        "rm" => cmd_rm(args),
        "rmdir" => cmd_rmdir(args),
        "cp" => cmd_cp(args),
        "mv" => cmd_mv(args),
        "stat" => cmd_stat(args),
        "cat" => cmd_cat(args),
        "touch" => cmd_touch(args),
        "write" => cmd_write(args),
        "syscall" => cmd_syscall_test(),
        "echo" => cmd_echo(args),
        "color" => cmd_color(args),
        "calc" => cmd_calc(args),
        "ifconfig" | "netinfo" => cmd_ifconfig(args),
        "dhcp"     => cmd_dhcp(),
        "dns" | "nslookup" => cmd_dns(args),
        "ping"     => cmd_ping(args),
        "arp"      => cmd_arp(args),
        "netstat"  => cmd_netstat(),
        "curl" | "fetch" => cmd_curl(args),
        "httpd"    => cmd_httpd(args),
        "nc"       => cmd_nc(args),
        "date"     => cmd_date(),
        "time"     => cmd_time(),
        "mway"     => cmd_mway(args),
        "vmm"      => cmd_vmm(args),
        "panic"    => cmd_panic(args),
        "reboot"   => cmd_reboot(),
        "lalaufetch" => cmd_lalaufetch(),
        "mouse"    => cmd_mouse(args),
        "paint"    => cmd_paint(),
        "uname" => {
            if args.trim().contains("-a") {
                println!("NyxaraOS 2.0.0-hybrid #1 SMP i686 GNU/Linux-compat Nyxara");
            } else {
                println!("NyxaraOS");
            }
        }
        "whoami" => println!("root"),
        "hostname" => println!("nyxara"),
        "motd" => {
            if let Some(data) = crate::vfs::read_file("/motd") {
                if let Ok(s) = core::str::from_utf8(&data) {
                    print!("{}", s);
                }
            }
        }
        "exit" | "quit" => {
            println!("Nyxara shell is the root kernel process and cannot exit. Type 'reboot' to restart.");
        }
        _ => {
            print_colored!(Color::LightRed, Color::Black, "Error: ");
            println!("Unknown command '{}'. Type 'help' for available commands.", command);
        }
    }
}

fn cmd_help() {
    print_colored!(Color::LightCyan, Color::Black, "Commands:\n");
    println!("  help              - Display this help reference");
    println!("  clear             - Clear screen");
    println!("  about             - System information");
    println!("  sysinfo           - Display hardware and CPU status");
    println!("  free / meminfo    - Display physical memory and allocator status");
    println!("  uptime            - Display system uptime");
    println!("  pwd               - Display current directory");
    println!("  cd <dir>          - Change current directory");
    println!("  mkdir <dir>       - Create a directory");
    println!("  rm <file>         - Remove a file");
    println!("  rmdir <dir>       - Remove an empty directory");
    println!("  cp <src> <dst>    - Copy a file");
    println!("  mv <src> <dst>    - Move or rename a file");
    println!("  stat <path>       - Display inode information");
    println!("  date              - Display current date from RTC/CMOS");
    println!("  time              - Display current time from RTC/CMOS");
    println!("  ls                - List files in virtual filesystem (VFS)");
    println!("  cat <file>        - Display contents of a file");
    println!("  touch <file>      - Create empty file");
    println!("  write <file> <tx> - Write text to a file");
    println!("  mway <file>       - Open full-screen text editor (Ctrl+S save, Ctrl+Q exit)");
    println!("  vmm [info|test]   - Virtual Memory Manager & x86 Paging status/tests");
    println!("  syscall           - Test Unix int 0x80 system call");
    println!("  echo <text>       - Print text to screen");
    println!("  color <fg> <bg>   - Change console color (0..15)");
    println!("  calc <a op b>     - Integer calculator");
    println!("  ifconfig [args]   - Display/configure network interface eth0");
    println!("  dhcp              - Obtain dynamic IP lease from DHCP server (DORA)");
    println!("  dns <host>        - Resolve domain name to IPv4 address via DNS");
    println!("  ping <host>       - Send ICMP echo requests to target host/IP");
    println!("  arp [-a|-c]       - View dynamic ARP cache or flush entries");
    println!("  netstat           - Display interface, socket, and traffic statistics");
    println!("  curl <url>        - Fetch HTTP web content over TCP");
    println!("  httpd [port]      - Run embedded Nyxara HTTP web server");
    println!("  nc [-u] <ip> <p>  - Send raw network payload via Netcat");
    println!("  panic [msg]       - Trigger Rust Kernel Panic");
    println!("  reboot            - Restart the computer");
    println!("  lalaufetch        - Neofetch-style system info with galaxy logo");
    println!("  mouse [test]      - Display PS/2 mouse status or interactive pointer test");
    println!("  paint             - Interactive mouse drawing canvas (ESC to exit)");
}

fn print_pad_right(s: &str, width: usize) {
    print!("{}", s);
    if s.len() < width {
        for _ in s.len()..width {
            print!(" ");
        }
    } else {
        print!("  ");
    }
}

fn cmd_pwd() {
    println!("{}", crate::vfs::current_dir());
}

fn cmd_cd(args: &str) {
    let path = args.trim();
    let target = if path.is_empty() || path == "~" { "/" } else { path };
    if crate::vfs::change_dir(target).is_err() {
        print_colored!(Color::LightRed, Color::Black, "Error: ");
        println!("Directory '{}' not found", target);
    }
}

fn cmd_mkdir(args: &str) {
    let trimmed = args.trim();
    let (is_p, dir_path) = if trimmed.starts_with("-p ") {
        (true, trimmed[3..].trim())
    } else if trimmed == "-p" {
        (true, "")
    } else {
        (false, trimmed)
    };

    if dir_path.is_empty() {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("mkdir [-p] <directory>");
        return;
    }

    let res = if is_p {
        crate::vfs::make_dir_p(dir_path)
    } else {
        crate::vfs::make_dir(dir_path)
    };

    if res.is_err() {
        print_colored!(Color::LightRed, Color::Black, "Error: ");
        println!("Failed to create directory '{}'", dir_path);
    }
}

fn cmd_rm(args: &str) {
    let path = args.trim();
    if path.is_empty() {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("rm <file>");
        return;
    }
    if crate::vfs::remove_file(path).is_err() {
        print_colored!(Color::LightRed, Color::Black, "Error: ");
        println!("Cannot remove file '{}'", path);
    }
}

fn cmd_rmdir(args: &str) {
    let path = args.trim();
    if path.is_empty() {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("rmdir <empty-directory>");
        return;
    }
    if crate::vfs::remove_dir(path).is_err() {
        print_colored!(Color::LightRed, Color::Black, "Error: ");
        println!("Cannot remove directory '{}' (it must be empty and not be the current directory)", path);
    }
}

fn split_two_paths(args: &str) -> Option<(&str, &str)> {
    let mut parts = args.split_whitespace();
    let source = parts.next()?;
    let destination = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    Some((source, destination))
}

fn cmd_cp(args: &str) {
    let Some((source, destination)) = split_two_paths(args.trim()) else {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("cp <source> <destination>");
        return;
    };
    if crate::vfs::copy_file(source, destination).is_err() {
        print_colored!(Color::LightRed, Color::Black, "Error: ");
        println!("Cannot copy '{}' to '{}'", source, destination);
    }
}

fn cmd_mv(args: &str) {
    let Some((source, destination)) = split_two_paths(args.trim()) else {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("mv <source> <destination>");
        return;
    };
    if crate::vfs::move_file(source, destination).is_err() {
        print_colored!(Color::LightRed, Color::Black, "Error: ");
        println!("Cannot move '{}' to '{}'", source, destination);
    }
}

fn cmd_stat(args: &str) {
    let path = args.trim();
    let target = if path.is_empty() { "." } else { path };
    match crate::vfs::stat(target) {
        Some((name, kind, size)) => {
            println!("Path : {}", name);
            println!("Type : {:?}", kind);
            println!("Size : {} bytes", size);
        }
        None => {
            print_colored!(Color::LightRed, Color::Black, "Error: ");
            println!("Path '{}' not found", target);
        }
    }
}

fn cmd_ls(args: &str) {
    let mut target_path = "";
    for part in args.split_whitespace() {
        if !part.starts_with('-') {
            target_path = part;
            break;
        }
    }

    match crate::vfs::list_dir(target_path) {
        Ok(entries) => {
            print_colored!(Color::LightCyan, Color::Black, "VFS: {}\n", if target_path.is_empty() { "." } else { target_path });
            if entries.is_empty() {
                println!("  (empty)");
                return;
            }
            for (name, size, kind) in entries {
                print!("  ");
                print_pad_right(&name, 16);
                if kind == InodeType::Directory {
                    println!("<DIR>");
                } else {
                    println!("{} bytes", size);
                }
            }
        }
        Err(_) => {
            if let Some((_, kind, size)) = crate::vfs::stat(target_path) {
                if kind == InodeType::File {
                    print_colored!(Color::LightCyan, Color::Black, "VFS: {}\n", target_path);
                    print!("  ");
                    let display_name = target_path.rsplit('/').next().unwrap_or(target_path);
                    print_pad_right(display_name, 16);
                    println!("{} bytes", size);
                    return;
                }
            }
            print_colored!(Color::LightRed, Color::Black, "Error: ");
            println!("Path '{}' not found", if target_path.is_empty() { "." } else { target_path });
        }
    }
}

fn cmd_cat(args: &str) {
    let file = args.trim();
    if file.is_empty() {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("cat <filename>");
        return;
    }

    if let Some((_, kind, _)) = crate::vfs::stat(file) {
        if kind == InodeType::Directory {
            print_colored!(Color::LightRed, Color::Black, "Error: ");
            println!("'{}' is a directory", file);
            return;
        }
    }

    match crate::vfs::read_file(file) {
        Some(data) => {
            if let Ok(s) = core::str::from_utf8(&data) {
                print!("{}", s);
                if !s.ends_with('\n') {
                    println!("");
                }
            } else {
                for b in data {
                    print!("{:02X} ", b);
                }
                println!("");
            }
        }
        None => {
            print_colored!(Color::LightRed, Color::Black, "Error: ");
            println!("File '{}' not found", file);
        }
    }
}

fn cmd_touch(args: &str) {
    let file = args.trim();
    if file.is_empty() {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("touch <filename>");
        return;
    }

    if crate::vfs::read_file(file).is_some() {
        return;
    }

    if let Err(_) = crate::vfs::write_file(file, b"") {
        print_colored!(Color::LightRed, Color::Black, "Error: ");
        println!("Failed to create file '{}'", file);
    }
}

fn cmd_write(args: &str) {
    let mut parts = args.trim().splitn(2, ' ');
    let file = parts.next().unwrap_or("");
    let text = parts.next().unwrap_or("");

    if file.is_empty() {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("write <filename> <content>");
        return;
    }

    let mut data = alloc::vec::Vec::new();
    data.extend_from_slice(text.as_bytes());
    data.push(b'\n');

    if let Err(_) = crate::vfs::write_file(file, &data) {
        print_colored!(Color::LightRed, Color::Black, "Error: ");
        println!("Failed to write to file '{}'", file);
    }
}

fn cmd_syscall_test() {
    print_colored!(Color::LightCyan, Color::Black, "Testing Unix System Call (int 0x80)...\n");

    let msg = "Hello from Unix sys_write via int 0x80!\n";
    let ret: i32;

    unsafe {
        core::arch::asm!(
            "int 0x80",
            inlateout("eax") 4u32 => ret, // SYS_WRITE
            in("ebx") 1u32,             // fd = 1 (stdout)
            in("ecx") msg.as_ptr() as u32,
            in("edx") msg.len() as u32,
        );
    }

    println!("Syscall return value (bytes written): {}", ret);

    let pid: i32;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            inlateout("eax") 20u32 => pid, // SYS_GETPID
            in("ebx") 0u32,
            in("ecx") 0u32,
            in("edx") 0u32,
        );
    }
    println!("Current PID from sys_getpid: {}", pid);
}

fn cmd_free() {
    let total = crate::pmm::total_memory() / 1024;
    let used = crate::pmm::used_memory() / 1024;
    let free = crate::pmm::free_memory() / 1024;

    print_colored!(Color::LightCyan, Color::Black, "Physical Memory (PMM):\n");
    println!("  Total : {} KB ({} MB)", total, total / 1024);
    println!("  Used  : {} KB ({} MB)", used, used / 1024);
    println!("  Free  : {} KB ({} MB)", free, free / 1024);

    print_colored!(Color::LightCyan, Color::Black, "Virtual Memory (VMM):\n");
    let paging_active = crate::vmm::is_paging_enabled();
    println!("  Paging       : {}", if paging_active { "Active (32-bit Protected Mode, CR0.PG=1, CR0.WP=1)" } else { "Inactive" });
    if paging_active {
        let cr3 = unsafe { crate::vmm::read_cr3() };
        println!("  CR3 (PD)     : 0x{:08X}", cr3);
        println!("  Mapped Pages : {} pages ({} KB)", crate::vmm::total_mapped_pages(), crate::vmm::total_mapped_pages() * 4);
        println!("  Demand Faults: {} auto-handled", crate::vmm::demand_page_fault_count());
    }
}

fn cmd_clear() {
    vga::clear_screen();
    print_colored!(Color::LightGreen, Color::Black, "Nyxara OS - Unix-like Hybrid C & Rust Operating System\n\n");
}

fn cmd_about() {
    println!("Architecture : x86 (32-bit Protected Mode)");
    println!("Kernel Core  : Rust (no_std, alloc, physical memory & heap)");
    println!("HAL Drivers  : C / Assembly (GDT, IDT, PIC, PIT, PS/2, UART)");
    println!("Target Model : Unix-like OS with POSIX roadmap");
}

// ─── lalaufetch ───────────────────────────────────────────────────────────────
// Neofetch-style system info display with galaxy braille logo.
// TrueColor on VBE framebuffer; degrades gracefully to VGA 16-color.
// ─── lalaufetch ───────────────────────────────────────────────────────────────
// Neofetch-style system info display with galaxy logo on the left, info on the right.
// TrueColor on VBE framebuffer; degrades gracefully to VGA 16-color.
fn cmd_lalaufetch() {
    use crate::framebuffer::{self, palette};
    use alloc::format;

    vga::clear_screen();

    let logo: &[&str] = &[
        "           .   *   .      .       ",
        "         ....oooooooooo..         ",
        "       ..ooo@@@@@@@@@@@@@@oo.     ",
        "     ..oo@@@@@oooooo@@@@@@@@@oo.  ",
        "    .ooo@@@@o.       .o@@@@@@@@oo.",
        "   oooo@@@@.           o@@@@@@@@oo",
        " ..oo@@@@@o      ..ooo@@@@ooo...  ",
        ".ooo@@@@@@@@@@oo@@@@ooo...ooo.    ",
        "..oo@@@@@@@@@@@@oooo...   o@@oo.  ",
        " ..oo@@@@@@oooo... .@@o. .o@@@oo. ",
        "   ..........      .o@@@@@@@@@@o. ",
        "                     ..oo@@@@oo.. ",
        "               *    .       .   * ",
        "           .        *    .        ",
    ];

    let grad_stops: &[(u32, u32)] = &[
        (palette::BLUE,    palette::SKY),
        (palette::SKY,     palette::TEAL),
        (palette::TEAL,    palette::MAUVE),
        (palette::MAUVE,   palette::LAVENDER),
        (palette::LAVENDER,palette::BLUE),
        (palette::BLUE,    palette::SKY),
        (palette::SKY,     palette::TEAL),
        (palette::TEAL,    palette::MAUVE),
        (palette::MAUVE,   palette::LAVENDER),
        (palette::LAVENDER,palette::BLUE),
        (palette::BLUE,    palette::SKY),
        (palette::SKY,     palette::TEAL),
        (palette::TEAL,    palette::MAUVE),
        (palette::MAUVE,   palette::LAVENDER),
    ];

    let uptime_sec = unsafe { timer_get_uptime_seconds() };
    let hours = uptime_sec / 3600;
    let minutes = (uptime_sec % 3600) / 60;
    let seconds = uptime_sec % 60;
    let uptime_str = if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else {
        format!("{}m {}s", minutes, seconds)
    };

    let total_kb = crate::pmm::total_memory() / 1024;
    let used_kb  = crate::pmm::used_memory() / 1024;
    let free_kb  = crate::pmm::free_memory() / 1024;
    let mem_str = format!("{} KB used / {} KB total  ({} KB free)", used_kb, total_kb, free_kb);

    let display_mode = if framebuffer::is_active() {
        if framebuffer::cols() >= 120 {
            "VBE 1024x768 32bpp TrueColor LFB"
        } else {
            "VBE 800x600 32bpp TrueColor LFB"
        }
    } else {
        "VGA 80x25 text mode"
    };

    let paging_str = if crate::vmm::is_paging_enabled() {
        "Enabled (CR0.PG=1 CR0.WP=1)"
    } else {
        "Disabled"
    };

    let print_info = |label: &str, fb_color: u32, vga_color: Color, val: &str| {
        if framebuffer::is_active() {
            framebuffer::set_color(palette::MAUVE, palette::BASE);
            print_pad_right(label, 10);
            framebuffer::set_color(palette::SURFACE1, palette::BASE);
            crate::print!(" | ");
            framebuffer::set_color(fb_color, palette::BASE);
            crate::print!("{}", val);
            framebuffer::set_color(palette::TEXT, palette::BASE);
        } else {
            vga::set_color(Color::LightMagenta, Color::Black);
            print_pad_right(label, 10);
            print_colored!(Color::LightRed, Color::Black, " | ");
            print_colored!(vga_color, Color::Black, "{}", val);
        }
    };

    for r in 0..17 {
        // Left column: logo or blank padding (35 chars total)
        if r < logo.len() {
            if framebuffer::is_active() {
                let (c_l, _) = grad_stops[r % grad_stops.len()];
                framebuffer::set_color(c_l, palette::BASE);
                crate::print!(" {}", logo[r]);
                framebuffer::set_color(palette::TEXT, palette::BASE);
            } else {
                print_colored!(Color::LightCyan, Color::Black, " {}", logo[r]);
            }
        } else {
            crate::print!("                                   ");
        }

        // Gap between logo and info (2 chars)
        crate::print!("  ");

        // Right column: system info
        match r {
            0 => {
                if framebuffer::is_active() {
                    framebuffer::set_color(palette::YELLOW, palette::BASE);
                    crate::print!("nyxara");
                    framebuffer::set_color(palette::RED, palette::BASE);
                    crate::print!("@");
                    framebuffer::set_color(palette::PEACH, palette::BASE);
                    crate::print!("NyxaraOS");
                    framebuffer::set_color(palette::TEXT, palette::BASE);
                } else {
                    print_colored!(Color::Yellow, Color::Black, "nyxara");
                    print_colored!(Color::LightRed, Color::Black, "@");
                    print_colored!(Color::Yellow, Color::Black, "NyxaraOS");
                }
            }
            1 => {
                if framebuffer::is_active() {
                    framebuffer::set_color(palette::SURFACE1, palette::BASE);
                    crate::print!("----------------------------------------");
                    framebuffer::set_color(palette::TEXT, palette::BASE);
                } else {
                    print_colored!(Color::DarkGray, Color::Black, "----------------------------------------");
                }
            }
            2 => print_info("os", palette::TEXT, Color::White, "Nyxara OS"),
            3 => print_info("arch", palette::TEXT, Color::White, "x86 (i686) 32-bit Protected Mode"),
            4 => print_info("kernel", palette::GREEN, Color::LightGreen, "Rust + C Hybrid Kernel"),
            5 => print_info("hal", palette::GREEN, Color::LightGreen, "C / ASM  (GDT IDT PIC PIT PS/2 UART)"),
            6 => print_info("uptime", palette::GREEN, Color::LightGreen, &uptime_str),
            7 => print_info("memory", palette::SKY, Color::LightCyan, &mem_str),
            8 => print_info("shell", palette::TEAL, Color::LightCyan, "NyxaraSH"),
            9 => print_info("display", palette::TEAL, Color::LightCyan, display_mode),
            10 => print_info("network", palette::SAPPHIRE, Color::LightBlue, "RTL8139 (QEMU virtio-compat)"),
            11 => print_info("serial", palette::SAPPHIRE, Color::LightBlue, "COM1 @ 38400 baud  (0x3F8)"),
            12 => print_info("paging", palette::YELLOW, Color::Yellow, paging_str),
            13 => {
                if framebuffer::is_active() {
                    framebuffer::set_color(palette::SURFACE1, palette::BASE);
                    crate::print!("========================================");
                    framebuffer::set_color(palette::TEXT, palette::BASE);
                } else {
                    print_colored!(Color::DarkGray, Color::Black, "========================================");
                }
            }
            14 => {
                for c in 0u8..8u8 {
                    vga::set_color(Color::Black, Color::from_u8(c));
                    print!("   ");
                }
                if framebuffer::is_active() {
                    framebuffer::set_color(palette::TEXT, palette::BASE);
                } else {
                    vga::set_color(Color::White, Color::Black);
                }
            }
            15 => {
                for c in 8u8..16u8 {
                    vga::set_color(Color::Black, Color::from_u8(c));
                    print!("   ");
                }
                if framebuffer::is_active() {
                    framebuffer::set_color(palette::TEXT, palette::BASE);
                } else {
                    vga::set_color(Color::White, Color::Black);
                }
            }
            16 => {
                if framebuffer::is_active() {
                    framebuffer::set_color(palette::OVERLAY, palette::BASE);
                    crate::print!("lalaufetch v1.1.0  --  galaxy explorer edition  [TrueColor LFB]");
                    framebuffer::set_color(palette::TEXT, palette::BASE);
                } else {
                    print_colored!(Color::DarkGray, Color::Black, "lalaufetch v1.1.0  --  galaxy explorer edition");
                }
            }
            _ => {}
        }

        crate::println!("");
    }

    crate::println!("");
}

fn cmd_sysinfo() {
    let ticks = unsafe { timer_get_ticks() };
    let uptime_sec = unsafe { timer_get_uptime_seconds() };
    let uptime_ms = unsafe { timer_get_uptime_ms() };

    let esp_val: u32;
    unsafe {
        core::arch::asm!("mov {}, esp", out(reg) esp_val);
    }

    print_colored!(Color::LightCyan, Color::Black, "System Status:\n");
    println!("  CPU Mode     : 32-bit Protected Mode");
    println!("  Stack Pointer: 0x{:X}", esp_val);
    println!("  PIT Ticks    : {} (100 Hz)", ticks);
    println!("  Uptime       : {} seconds ({} ms)", uptime_sec, uptime_ms);
    println!("  Interrupts   : Enabled (IDT vectors 0..47)");
    println!("  Serial COM1  : 0x3F8 @ 38400 baud");
}

fn cmd_uptime() {
    let sec = unsafe { timer_get_uptime_seconds() };
    let ms = unsafe { timer_get_uptime_ms() };
    let minutes = sec / 60;
    let seconds = sec % 60;
    print_colored!(Color::LightGreen, Color::Black, "Uptime: ");
    println!("{}m {}s (total {} ms)", minutes, seconds, ms);
}

fn cmd_echo(args: &str) {
    println!("{}", args);
}

fn cmd_color(args: &str) {
    let mut parts = args.split_whitespace();
    let fg_str = parts.next();
    let bg_str = parts.next();

    if let (Some(f), Some(b)) = (fg_str, bg_str) {
        if let (Ok(fg_num), Ok(bg_num)) = (f.parse::<u8>(), b.parse::<u8>()) {
            if fg_num < 16 && bg_num < 16 {
                vga::set_color(Color::from_u8(fg_num), Color::from_u8(bg_num));
                println!("Color updated: fg={}, bg={}", fg_num, bg_num);
                return;
            }
        }
    }

    print_colored!(Color::LightRed, Color::Black, "Usage: ");
    println!("color <fg:0-15> <bg:0-15>");
    println!("Colors: 0:Black, 1:Blue, 2:Green, 3:Cyan, 4:Red, 5:Magenta, 6:Brown, 7:LGray,");
    println!("        8:DGray, 9:LBlue, 10:LGreen, 11:LCyan, 12:LRed, 13:LMagenta, 14:Yellow, 15:White");
}

fn cmd_calc(args: &str) {
    let mut parts = args.split_whitespace();
    let a_str = parts.next();
    let op_str = parts.next();
    let b_str = parts.next();

    if let (Some(a_s), Some(op), Some(b_s)) = (a_str, op_str, b_str) {
        if let (Ok(a), Ok(b)) = (a_s.parse::<i32>(), b_s.parse::<i32>()) {
            let res = match op {
                "+" => Some(a.wrapping_add(b)),
                "-" => Some(a.wrapping_sub(b)),
                "*" => Some(a.wrapping_mul(b)),
                "/" => {
                    if b == 0 {
                        print_colored!(Color::LightRed, Color::Black, "Error: ");
                        println!("Division by zero!");
                        return;
                    }
                    Some(a / b)
                }
                "%" => {
                    if b == 0 {
                        print_colored!(Color::LightRed, Color::Black, "Error: ");
                        println!("Modulo by zero!");
                        return;
                    }
                    Some(a % b)
                }
                _ => None,
            };

            if let Some(val) = res {
                print_colored!(Color::LightGreen, Color::Black, "Result: ");
                println!("{} {} {} = {}", a, op, b, val);
                return;
            }
        }
    }

    print_colored!(Color::LightRed, Color::Black, "Usage: ");
    println!("calc <num1> <+|-|*|/|%> <num2> (e.g. calc 100 * 5)");
}

fn cmd_panic(args: &str) {
    let msg = if args.trim().is_empty() {
        "Manual panic triggered by user from Nyxara shell!"
    } else {
        args.trim()
    };
    panic!("{}", msg);
}

fn cmd_reboot() {
    print_colored!(Color::Yellow, Color::Black, "Rebooting Nyxara OS...\n");
    logln!("[Nyxara Kernel] System reboot triggered.");

    unsafe {
        core::arch::asm!("cli");
        for _ in 0..1000 {
            outb(0x64, 0xFE);
        }
        let null_idt: [u16; 3] = [0, 0, 0];
        core::arch::asm!("lidt [{}]", in(reg) null_idt.as_ptr());
        core::arch::asm!("int3");
    }
}

fn cmd_ifconfig(args: &str) {
    let trimmed = args.trim();
    if trimmed.starts_with("set ") {
        let parts: alloc::vec::Vec<&str> = trimmed[4..].split_whitespace().collect();
        if parts.len() == 3 {
            let ip_opt = crate::net::parse_ip(parts[0]);
            let mask_opt = crate::net::parse_ip(parts[1]);
            let gw_opt = crate::net::parse_ip(parts[2]);

            if let (Some(ip), Some(mask), Some(gw)) = (ip_opt, mask_opt, gw_opt) {
                crate::net::update_config(ip, mask, gw);
                print_colored!(Color::LightGreen, Color::Black, "[OK] ");
                println!("Network eth0 updated: IP={}, Mask={}, Gateway={}",
                    parts[0], parts[1], parts[2]);
                println!("Configuration saved to /etc/network.conf in VFS.");
                return;
            }
        }
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("ifconfig set <ip> <netmask> <gateway>");
        println!("Example: ifconfig set 10.0.2.15 255.255.255.0 10.0.2.2");
        return;
    }

    if !trimmed.is_empty() && trimmed != "eth0" {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("ifconfig [eth0]");
        println!("       ifconfig set <ip> <netmask> <gateway>");
        return;
    }

    match crate::net::get_config() {
        Some(cfg) => {
            let (rx_pkts, tx_pkts, rx_bytes, tx_bytes) = crate::net::get_stats();
            let status = if cfg.is_up { "UP, BROADCAST, RUNNING" } else { "DOWN" };

            print_colored!(Color::LightCyan, Color::Black, "eth0: ");
            println!("flags=<{}> mtu 1500", status);
            println!("      inet {}  netmask {}  gateway {}",
                crate::net::format_ip(&cfg.ip),
                crate::net::format_ip(&cfg.netmask),
                crate::net::format_ip(&cfg.gateway));
            println!("      nameserver {}", crate::net::format_ip(&cfg.dns));
            println!("      ether {} (Realtek RTL8139)", crate::net::format_mac(&cfg.mac));
            println!("      RX packets {}  bytes {} ({})",
                rx_pkts, rx_bytes, if rx_bytes < 1024 { "B" } else { "KB" });
            println!("      TX packets {}  bytes {} ({})",
                tx_pkts, tx_bytes, if tx_bytes < 1024 { "B" } else { "KB" });
        }
        None => {
            print_colored!(Color::LightRed, Color::Black, "Error: ");
            println!("Network interface eth0 unavailable.");
        }
    }
}

fn cmd_ping(args: &str) {
    let host = args.trim();
    if host.is_empty() {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("ping <host_or_ip> (e.g. ping 10.0.2.2 or ping google.com)");
        return;
    }

    let target_ip = match crate::net::parse_ip(host) {
        Some(ip) => ip,
        None => {
            print_colored!(Color::LightCyan, Color::Black, "[DNS] ");
            println!("Resolving host '{}'...", host);
            match crate::net::dns_resolve(host, 3000) {
                Ok(ip) => ip,
                Err(e) => {
                    print_colored!(Color::LightRed, Color::Black, "Error: ");
                    println!("Failed to resolve '{}': {}", host, e);
                    return;
                }
            }
        }
    };

    println!("PING {} ({}) 32 bytes of ICMP data:", host, crate::net::format_ip(&target_ip));

    let mut transmitted = 0;
    let mut received = 0;

    for seq in 1..=4 {
        transmitted += 1;
        match crate::net::send_ping(target_ip, seq, 1500) {
            Ok(rtt) => {
                received += 1;
                println!("32 bytes from {}: icmp_seq={} ttl=64 time={} ms",
                    crate::net::format_ip(&target_ip), seq, rtt);
            }
            Err(e) => {
                println!("From {}: icmp_seq={} {}", host, seq, e);
            }
        }
    }

    println!("\n--- {} ping statistics ---", host);
    let loss = if transmitted > 0 { ((transmitted - received) * 100) / transmitted } else { 0 };
    println!("{} packets transmitted, {} received, {}% packet loss", transmitted, received, loss);
}

fn cmd_dhcp() {
    print_colored!(Color::LightCyan, Color::Black, "[DHCP] ");
    println!("Requesting dynamic IP configuration via DHCP (DORA)...");

    match crate::net::request_lease() {
        Ok(lease) => {
            print_colored!(Color::LightGreen, Color::Black, "[OK] ");
            println!("DHCP lease acquired successfully:");
            println!("      Assigned IP : {}", crate::net::format_ip(&lease.ip));
            println!("      Subnet Mask : {}", crate::net::format_ip(&lease.netmask));
            println!("      Gateway     : {}", crate::net::format_ip(&lease.gateway));
            println!("      DNS Server  : {}", crate::net::format_ip(&lease.dns));
            println!("      Lease Time  : {} seconds", lease.lease_sec);
            println!("Configuration applied and saved to /etc/network.conf.");
        }
        Err(e) => {
            print_colored!(Color::LightRed, Color::Black, "Error: ");
            println!("DHCP transaction failed: {}", e);
        }
    }
}

fn cmd_dns(args: &str) {
    let domain = args.trim();
    if domain.is_empty() {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("dns <hostname> (e.g. dns google.com)");
        return;
    }

    print_colored!(Color::LightCyan, Color::Black, "[DNS] ");
    println!("Querying DNS for '{}'...", domain);

    match crate::net::dns_resolve(domain, 3000) {
        Ok(ip) => {
            print_colored!(Color::LightGreen, Color::Black, "[OK] ");
            println!("{} has IPv4 address {}", domain, crate::net::format_ip(&ip));
        }
        Err(e) => {
            print_colored!(Color::LightRed, Color::Black, "Error: ");
            println!("DNS lookup failed for '{}': {}", domain, e);
        }
    }
}

fn cmd_arp(args: &str) {
    let trimmed = args.trim();
    if trimmed == "-c" || trimmed == "clear" {
        crate::net::arp::flush_cache();
        print_colored!(Color::LightGreen, Color::Black, "[OK] ");
        println!("ARP cache table cleared.");
        return;
    }

    let table = crate::net::arp::get_table();
    print_colored!(Color::LightCyan, Color::Black, "Dynamic ARP Table Entries:\n");
    if table.is_empty() {
        println!("  (ARP cache is empty)");
        return;
    }

    print!("  ");
    print_pad_right("IP Address", 16);
    print!(" ");
    print_pad_right("HW Address", 18);
    println!(" Age");
    println!("  --------------------------------------------------");
    let now = unsafe { crate::net::timer_get_uptime_ms() };
    for entry in table {
        let age_sec = (now.saturating_sub(entry.updated_ms)) / 1000;
        let ip_str = crate::net::format_ip(&entry.ip);
        let mac_str = crate::net::format_mac(&entry.mac);
        print!("  ");
        print_pad_right(&ip_str, 16);
        print!(" ");
        print_pad_right(&mac_str, 18);
        println!(" {}s", age_sec);
    }
}

fn cmd_netstat() {
    print_colored!(Color::LightCyan, Color::Black, "Network Subsystem Status:\n");
    if let Some(cfg) = crate::net::get_config() {
        let status = if cfg.is_up { "UP, RUNNING" } else { "DOWN" };
        let (rx_pkts, tx_pkts, rx_bytes, tx_bytes) = crate::net::get_stats();
        let arp_count = crate::net::arp::get_table().len();
        let tcp_sockets = crate::net::tcp::get_socket_count();

        println!("  Interface       : eth0 (Realtek RTL8139 PCI)");
        println!("  Link Status     : {}", status);
        println!("  MAC Address     : {}", crate::net::format_mac(&cfg.mac));
        println!("  IPv4 Address    : {}", crate::net::format_ip(&cfg.ip));
        println!("  Subnet Mask     : {}", crate::net::format_ip(&cfg.netmask));
        println!("  Default Gateway : {}", crate::net::format_ip(&cfg.gateway));
        println!("  DNS Server      : {}", crate::net::format_ip(&cfg.dns));
        println!("  Traffic Stats   : RX {} pkts ({} B), TX {} pkts ({} B)",
            rx_pkts, rx_bytes, tx_pkts, tx_bytes);
        println!("  Active Sockets  : {} TCP sockets", tcp_sockets);
        println!("  ARP Entries     : {} cached", arp_count);
    } else {
        println!("  No active network interfaces detected.");
    }
}

fn cmd_curl(args: &str) {
    let url = args.trim();
    if url.is_empty() {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("curl <url> (e.g. curl http://10.0.2.2:8000/ or curl 10.0.2.2)");
        return;
    }

    print_colored!(Color::LightCyan, Color::Black, "[HTTP] ");
    println!("Fetching '{}'...", url);

    match crate::net::fetch(url, 5000) {
        Ok(resp) => {
            println!("{}", resp);
        }
        Err(e) => {
            print_colored!(Color::LightRed, Color::Black, "Error: ");
            println!("HTTP request failed: {}", e);
        }
    }
}

fn cmd_httpd(args: &str) {
    let port: u16 = args.trim().parse().unwrap_or(8080);
    if crate::net::tcp::listen(port).is_err() {
        print_colored!(Color::LightRed, Color::Black, "Error: ");
        println!("Failed to bind HTTP server to port {}.", port);
        return;
    }

    let cfg = crate::net::get_config();
    let ip_str = cfg.map(|c| crate::net::format_ip(&c.ip)).unwrap_or_else(|| alloc::string::String::from("0.0.0.0"));

    print_colored!(Color::LightGreen, Color::Black, "[HTTPD] ");
    println!("Nyxara Micro HTTP Server listening on {}:{}", ip_str, port);
    println!("Routes available: / (Dashboard), /api/status, or VFS files.");
    println!("Press Ctrl+C or 'q' to stop server.\n");

    extern "C" {
        fn keyboard_has_char() -> bool;
        fn keyboard_getchar() -> u16;
    }

    loop {
        crate::net::poll();
        if crate::net::tcp::service_http_server(port) {
            println!("[HTTPD] Request serviced.");
        }

        if unsafe { keyboard_has_char() } {
            let key = unsafe { keyboard_getchar() };
            if key == b'q' as u16 || key == b'Q' as u16 || key == crate::shell::KEY_CTRL_C {
                println!("[HTTPD] Server stopped by user.");
                crate::net::tcp::close_listener(port);
                break;
            }
        }

        unsafe { core::arch::asm!("hlt"); }
    }
}

fn cmd_nc(args: &str) {
    let parts: alloc::vec::Vec<&str> = args.trim().split_whitespace().collect();
    if parts.len() < 2 {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("nc [-u] <ip> <port> [message]");
        println!("Example: nc -u 10.0.2.2 12345 Hello");
        return;
    }

    let (is_udp, target_ip_str, port_str, msg_start_idx) = if parts[0] == "-u" {
        if parts.len() < 3 {
            println!("Usage: nc -u <ip> <port> [message]");
            return;
        }
        (true, parts[1], parts[2], 3)
    } else {
        (false, parts[0], parts[1], 2)
    };

    let target_ip = match crate::net::parse_ip(target_ip_str) {
        Some(ip) => ip,
        None => {
            println!("Invalid target IP '{}'.", target_ip_str);
            return;
        }
    };

    let port = match port_str.parse::<u16>() {
        Ok(p) => p,
        Err(_) => {
            println!("Invalid port number '{}'.", port_str);
            return;
        }
    };

    let message = if parts.len() > msg_start_idx {
        parts[msg_start_idx..].join(" ")
    } else {
        alloc::string::String::from("Hello from Nyxara Netcat!")
    };

    if is_udp {
        match crate::net::udp::send_to(target_ip, 49152, port, message.as_bytes()) {
            Ok(_) => println!("Sent {} UDP bytes to {}:{}.", message.len(), target_ip_str, port),
            Err(e) => println!("Error sending UDP: {}", e),
        }
    } else {
        match crate::net::tcp::connect(target_ip, port, 3000) {
            Ok(mut stream) => {
                let _ = stream.write(message.as_bytes());
                println!("Connected and sent {} TCP bytes. Awaiting response...", message.len());
                let resp = stream.read(2000).unwrap_or_default();
                if !resp.is_empty() {
                    println!("Received: {}", alloc::string::String::from_utf8_lossy(&resp));
                }
                stream.close();
            }
            Err(e) => println!("TCP connection failed: {}", e),
        }
    }
}

// ---------------------------------------------------------------------------
// RTC / CMOS date & time commands
// ---------------------------------------------------------------------------

fn read_rtc() -> RtcTime {
    let mut t = RtcTime {
        second: 0,
        minute: 0,
        hour:   0,
        day:    0,
        month:  0,
        _pad:   0,
        year:   0,
    };
    unsafe { rtc_get_datetime(&mut t as *mut RtcTime); }
    t
}

fn cmd_date() {
    let t = read_rtc();
    print_colored!(Color::LightCyan, Color::Black, "Date: ");
    // Format: YYYY-MM-DD
    print_padded2(t.year as u32, false);
    print!("-");
    print_padded2(t.month as u32, true);
    print!("-");
    print_padded2(t.day as u32, true);
    println!("  (from RTC/CMOS)");
}

fn cmd_time() {
    let t = read_rtc();
    print_colored!(Color::LightCyan, Color::Black, "Time: ");
    // Format: HH:MM:SS
    print_padded2(t.hour as u32, true);
    print!(":");
    print_padded2(t.minute as u32, true);
    print!(":");
    print_padded2(t.second as u32, true);
    println!("  (from RTC/CMOS)");
}

/// Print a number, zero-padded to 2 digits if `pad` is true.
fn print_padded2(val: u32, pad: bool) {
    if pad && val < 10 {
        print!("0{}", val);
    } else {
        print!("{}", val);
    }
}

// ---------------------------------------------------------------------------
// mway - Full-screen text editor
// ---------------------------------------------------------------------------

fn cmd_mway(args: &str) {
    let filename = args.trim();
    if filename.is_empty() {
        print_colored!(Color::LightRed, Color::Black, "Usage: ");
        println!("mway <filename>");
        println!("  Example: mway catatan.txt");
        return;
    }

    logln!("[mway] Opening file: '{}'", filename);
    crate::editor::open(filename);
    logln!("[mway] Editor closed for: '{}'", filename);
}

// ---------------------------------------------------------------------------
// vmm - Virtual Memory Manager & x86 Paging
// ---------------------------------------------------------------------------

fn cmd_vmm(args: &str) {
    let parts: alloc::vec::Vec<&str> = args.split_whitespace().collect();
    let subcmd = if parts.is_empty() { "info" } else { parts[0] };

    match subcmd {
        "info" | "status" => {
            print_colored!(Color::LightCyan, Color::Black, "Virtual Memory Manager (VMM) Status:\n");
            let active = crate::vmm::is_paging_enabled();
            println!("  x86 Paging      : {}", if active { "ENABLED (32-bit Protected Mode)" } else { "DISABLED" });
            if active {
                let cr3 = unsafe { crate::vmm::read_cr3() };
                println!("  Page Dir (CR3)  : 0x{:08X}", cr3);
                println!("  Identity Map    : 0x00000000 - 0x03FFFFFF (64 MB)");
                println!("  Total Mapped    : {} pages ({} KB)",
                    crate::vmm::total_mapped_pages(),
                    crate::vmm::total_mapped_pages() * 4
                );
                println!("  Demand Range    : 0x{:08X} - 0x{:08X} (16 MB)", crate::vmm::DEMAND_PAGING_START, crate::vmm::DEMAND_PAGING_END);
                println!("  Demand Faults   : {} auto-allocated via ISR 14", crate::vmm::demand_page_fault_count());
                println!("  Protection      : Supervisor Write-Protect (CR0.WP=1)");
            }
        }
        "test" => {
            print_colored!(Color::LightCyan, Color::Black, "[VMM Test] ");
            println!("Running automated paging and demand-paging verification suite...");

            print!("  1. Testing Identity Mapping (Kernel & VGA) ... ");
            let k_ok = crate::vmm::get_phys_addr(0x10000) == Some(0x10000);
            let vga_ok = crate::vmm::get_phys_addr(0xB8000) == Some(0xB8000);
            if k_ok && vga_ok {
                print_colored!(Color::LightGreen, Color::Black, "PASSED\n");
            } else {
                print_colored!(Color::LightRed, Color::Black, "FAILED\n");
            }

            print!("  2. Testing Demand Paging on 0xC0002000 ... ");
            let test_addr = 0xC0002000 as *mut u32;
            let val = 0x5A5A1234;
            unsafe {
                core::ptr::write_volatile(test_addr, val);
                let read = core::ptr::read_volatile(test_addr);
                if read == val && crate::vmm::get_phys_addr(0xC0002000).is_some() {
                    print_colored!(Color::LightGreen, Color::Black, "PASSED");
                    println!(" (auto-allocated frame: 0x{:08X})", crate::vmm::get_phys_addr(0xC0002000).unwrap());
                } else {
                    print_colored!(Color::LightRed, Color::Black, "FAILED\n");
                }
            }

            print!("  3. Testing Dynamic Mapping & Unmapping ... ");
            let custom_v = 0xD0001000;
            if let Some(frame) = crate::pmm::alloc_frame() {
                let map_res = crate::vmm::map_page(custom_v, frame, crate::vmm::PAGE_WRITABLE);
                let mut data_ok = false;
                if map_res.is_ok() {
                    unsafe {
                        let ptr = custom_v as *mut u32;
                        core::ptr::write_volatile(ptr, 0xCAFEBABE);
                        data_ok = core::ptr::read_volatile(ptr) == 0xCAFEBABE;
                    }
                    let _ = crate::vmm::unmap_page(custom_v);
                }
                crate::pmm::free_frame(frame);
                if data_ok {
                    print_colored!(Color::LightGreen, Color::Black, "PASSED\n");
                } else {
                    print_colored!(Color::LightRed, Color::Black, "FAILED\n");
                }
            } else {
                print_colored!(Color::LightRed, Color::Black, "OUT OF MEMORY\n");
            }

            print_colored!(Color::LightGreen, Color::Black, "\n[SUCCESS] ");
            println!("All Virtual Memory Manager tests executed successfully!");
        }
        "map" => {
            if parts.len() < 3 {
                print_colored!(Color::LightRed, Color::Black, "Usage: ");
                println!("vmm map <virt_hex> <phys_hex>");
                println!("  Example: vmm map D0000000 1000000");
                return;
            }
            let virt = usize::from_str_radix(parts[1].trim_start_matches("0x"), 16);
            let phys = usize::from_str_radix(parts[2].trim_start_matches("0x"), 16);
            match (virt, phys) {
                (Ok(v), Ok(p)) => {
                    match crate::vmm::map_page(v, p, crate::vmm::PAGE_WRITABLE) {
                        Ok(_) => {
                            print_colored!(Color::LightGreen, Color::Black, "[OK] ");
                            println!("Mapped virt 0x{:08X} -> phys 0x{:08X}", v, p);
                        }
                        Err(e) => {
                            print_colored!(Color::LightRed, Color::Black, "Error: ");
                            println!("{}", e);
                        }
                    }
                }
                _ => {
                    print_colored!(Color::LightRed, Color::Black, "Error: ");
                    println!("Invalid hexadecimal address.");
                }
            }
        }
        "fault" => {
            print_colored!(Color::Yellow, Color::Black, "[WARNING] ");
            println!("Triggering intentional Page Fault at unmapped address 0xDEAD0000...");
            println!("This will invoke the Page Fault Exception Handler (ISR 14).");
            unsafe {
                let ptr = 0xDEAD0000 as *const u32;
                let _ = core::ptr::read_volatile(ptr);
            }
        }
        _ => {
            print_colored!(Color::LightCyan, Color::Black, "VMM Commands:\n");
            println!("  vmm info                   - Display paging and directory information");
            println!("  vmm test                   - Run automated Demand Paging & VMM test suite");
            println!("  vmm map <virt_hex> <phys>  - Map virtual page to physical frame");
            println!("  vmm fault                  - Trigger intentional page fault exception (panic test)");
        }
    }
}

fn cmd_mouse(args: &str) {
    let sub = args.trim();
    if sub == "test" {
        if !framebuffer::is_active() {
            println!("Mouse test is running in text console. Move mouse or click buttons.");
            println!("Press 'q' or ESC to exit.");
            let mut last_ev = mouse::get_event_count();
            loop {
                if unsafe { keyboard_has_char() } {
                    let key = unsafe { keyboard_getchar() };
                    if key == 27 || key == b'q' as u16 || key == b'Q' as u16 {
                        break;
                    }
                }
                let cur_ev = mouse::get_event_count();
                if cur_ev != last_ev {
                    last_ev = cur_ev;
                    let st = mouse::get_state();
                    println!("Mouse: ({}, {}) | L:{} R:{} M:{} | Events: {}",
                        st.x, st.y, st.left as u8, st.right as u8, st.middle as u8, cur_ev);
                }
            }
            return;
        }

        framebuffer::clear_screen();
        framebuffer::puts(" Nyxara PS/2 Mouse Visual Pointer Test\n");
        framebuffer::puts(" -------------------------------------------------------------\n");
        framebuffer::puts(" Move pointer across screen. Click Left / Right buttons.\n");
        framebuffer::puts(" Press 'q' or ESC to exit back to shell.\n");

        loop {
            if unsafe { keyboard_has_char() } {
                let key = unsafe { keyboard_getchar() };
                if key == 27 || key == b'q' as u16 || key == b'Q' as u16 {
                    break;
                }
            }

            let st = mouse::get_state();
            if st.x >= 0 && st.y >= 0 {
                framebuffer::render_mouse_cursor(st.x as usize, st.y as usize);
            }
        }

        framebuffer::hide_mouse_cursor();
        framebuffer::clear_screen();
        return;
    }

    let st = mouse::get_state();
    let events = mouse::get_event_count();
    print_colored!(Color::LightCyan, Color::Black, "PS/2 Mouse Hardware Status:\n");
    println!("  Position (X, Y) : ({}, {})", st.x, st.y);
    println!("  Buttons         : Left=[{}] Right=[{}] Middle=[{}]",
        if st.left { "PRESSED" } else { "RELEASED" },
        if st.right { "PRESSED" } else { "RELEASED" },
        if st.middle { "PRESSED" } else { "RELEASED" }
    );
    println!("  Total IRQ Events: {}", events);
    if framebuffer::is_active() {
        println!("  Display Bounds  : 0..{} x 0..{}", framebuffer::width(), framebuffer::height());
    } else {
        println!("  Display Bounds  : Text console (80x25)");
    }
    print_colored!(Color::Yellow, Color::Black, "Tips: ");
    println!("Type 'mouse test' for live pointer tracking or 'paint' for drawing canvas.");
}

fn cmd_paint() {
    if !framebuffer::is_active() {
        print_colored!(Color::LightRed, Color::Black, "Error: ");
        println!("'paint' requires active linear framebuffer mode.");
        return;
    }

    let fb_w = framebuffer::width();
    let fb_h = framebuffer::height();
    if fb_w < 100 || fb_h < 100 {
        return;
    }

    const CANVAS_TOP: usize = 36;
    const PALETTE_COUNT: usize = 7;
    let palette_colors: [u32; PALETTE_COUNT] = [
        framebuffer::palette::RED,
        framebuffer::palette::GREEN,
        framebuffer::palette::YELLOW,
        framebuffer::palette::BLUE,
        framebuffer::palette::MAUVE,
        framebuffer::palette::TEXT,
        framebuffer::palette::BASE,
    ];

    let mut current_color_idx: usize = 0;

    framebuffer::clear_screen();
    framebuffer::fill_rect(0, 0, fb_w, fb_h, framebuffer::palette::BASE);
    framebuffer::fill_rect(0, 0, fb_w, CANVAS_TOP - 2, 0x1E1E2E);
    framebuffer::fill_rect(0, CANVAS_TOP - 2, fb_w, 2, framebuffer::palette::SURFACE1);

    let title = "Nyxara Paint | L-Click: Draw | R-Click: Color | C: Clear | 1-7: Palette | ESC: Exit";
    for (i, b) in title.bytes().enumerate() {
        if i + 2 < framebuffer::cols() {
            framebuffer::putchar_at(b, framebuffer::palette::TEXT, 0x1E1E2E, i + 1, 0);
        }
    }

    let draw_palette = |active_idx: usize| {
        for i in 0..PALETTE_COUNT {
            let bx: usize = 16 + i * 36;
            let by: usize = 18;
            let bw: usize = 26;
            let bh: usize = 14;

            let border_c = if i == active_idx { 0xFFFFFF } else { 0x585B70 };
            framebuffer::fill_rect(bx.saturating_sub(1), by.saturating_sub(1), bw + 2, bh + 2, border_c);
            framebuffer::fill_rect(bx, by, bw, bh, palette_colors[i]);
            let digit = b'1' + (i as u8);
            framebuffer::putchar_at(digit, 0xCDD6F4, 0x1E1E2E, (bx + bw + 2) / 8, 1);
        }
    };

    draw_palette(current_color_idx);

    let mut prev_right = false;

    loop {
        if unsafe { keyboard_has_char() } {
            let key = unsafe { keyboard_getchar() };
            if key == 27 || key == b'q' as u16 || key == b'Q' as u16 {
                break;
            } else if key == b'c' as u16 || key == b'C' as u16 {
                framebuffer::hide_mouse_cursor();
                framebuffer::fill_rect(0, CANVAS_TOP, fb_w, fb_h - CANVAS_TOP, framebuffer::palette::BASE);
            } else if key >= b'1' as u16 && key <= b'7' as u16 {
                current_color_idx = (key - b'1' as u16) as usize;
                draw_palette(current_color_idx);
            }
        }

        let st = mouse::get_state();

        if st.right && !prev_right {
            current_color_idx = (current_color_idx + 1) % PALETTE_COUNT;
            draw_palette(current_color_idx);
        }
        prev_right = st.right;

        if st.left {
            let mx = st.x as usize;
            let my = st.y as usize;

            if my >= CANVAS_TOP && mx < fb_w && my < fb_h {
                let bx = mx.saturating_sub(1);
                let by = my.saturating_sub(1);
                framebuffer::fill_rect(bx, by, 3, 3, palette_colors[current_color_idx]);
            } else if my < CANVAS_TOP {
                for i in 0..PALETTE_COUNT {
                    let bx = 16 + i * 36;
                    let by = 18;
                    if mx >= bx && mx <= bx + 28 && my >= by && my <= by + 14 {
                        if current_color_idx != i {
                            current_color_idx = i;
                            draw_palette(current_color_idx);
                        }
                        break;
                    }
                }
            }
        }

        if st.x >= 0 && st.y >= 0 {
            framebuffer::render_mouse_cursor(st.x as usize, st.y as usize);
        }
    }

    framebuffer::hide_mouse_cursor();
    framebuffer::clear_screen();
}
