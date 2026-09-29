use alloc::string::String;
use core::arch::asm;

fn cpuid(leaf: u32) -> [u32; 4] {
    let (a, b, c, d);
    unsafe {
        asm!(
            "cpuid",
            inlateout("eax") leaf => a,
            out("ebx") b,
            out("ecx") c,
            out("edx") d,
        );
    }
    [a, b, c, d]
}

pub fn available() -> bool {
    // CPUID is supported when the EFLAGS ID bit (21) can be toggled.
    let (orig, probe): (u32, u32);
    unsafe {
        asm!(
            "pushfd",
            "pop {orig}",
            "mov {probe}, {orig}",
            "btc {probe}, 21",
            "push {probe}",
            "popfd",
            "pushfd",
            "pop {probe}",
            "push {orig}",
            "popfd",
            orig = out(reg) orig,
            probe = out(reg) probe,
        );
    }
    (probe >> 21) & 1 == 1
}

pub fn vendor_string() -> Option<String> {
    if !available() {
        return None;
    }
    let v = cpuid(0);
    let mut bytes = [0u8; 12];
    bytes[0..4].copy_from_slice(&v[1].to_le_bytes());
    bytes[4..8].copy_from_slice(&v[3].to_le_bytes());
    bytes[8..12].copy_from_slice(&v[2].to_le_bytes());
    Some(String::from_utf8_lossy(&bytes).trim().into())
}

pub fn brand_string() -> Option<String> {
    if !available() {
        return None;
    }
    let max_ext = cpuid(0x8000_0000)[0];
    if max_ext < 0x8000_0004 {
        return None;
    }
    let mut bytes = [0u8; 48];
    for (i, leaf) in (0x8000_0002u32..=0x8000_0004).enumerate() {
        let r = cpuid(leaf);
        for (j, reg) in r.iter().enumerate() {
            bytes[i * 16 + j * 4..i * 16 + j * 4 + 4].copy_from_slice(&reg.to_le_bytes());
        }
    }
    let brand = String::from_utf8_lossy(&bytes).trim().into();
    Some(brand)
}
