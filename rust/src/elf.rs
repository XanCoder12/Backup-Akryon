use crate::vmm::{self, PAGE_SIZE, PAGE_USER, PAGE_WRITABLE};

const ELF32_HEADER_SIZE: usize = 52;
const PROGRAM_HEADER_SIZE: usize = 32;
const PT_LOAD: u32 = 1;
const PF_W: u32 = 1 << 1;
const ENOEXEC: i32 = -8;

fn u16_at(image: &[u8], off: usize) -> Option<u16> {
    let bytes = image.get(off..off + 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn u32_at(image: &[u8], off: usize) -> Option<u32> {
    let bytes = image.get(off..off + 4)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// Load an ELF32 i386 executable image into a page directory and return
/// the entry point. Frames are filled through the identity map, so the
/// target directory does not need to be active.
pub fn load(pd_phys: usize, image: &[u8]) -> Result<usize, i32> {
    let header = parse_header(image)?;
    let entry = header.entry;

    for i in 0..header.phnum {
        let ph = header.phoff + i * header.phentsize;
        if u32_at(image, ph).ok_or(ENOEXEC)? != PT_LOAD {
            continue;
        }
        let offset = u32_at(image, ph + 4).ok_or(ENOEXEC)? as usize;
        let vaddr = u32_at(image, ph + 8).ok_or(ENOEXEC)? as usize;
        let filesz = u32_at(image, ph + 16).ok_or(ENOEXEC)? as usize;
        let memsz = u32_at(image, ph + 20).ok_or(ENOEXEC)? as usize;
        let flags = u32_at(image, ph + 24).ok_or(ENOEXEC)?;
        if vaddr < vmm::USER_SPACE_START || vaddr + memsz > vmm::USER_SPACE_END {
            return Err(ENOEXEC);
        }
        if offset + filesz > image.len() {
            return Err(ENOEXEC);
        }
        load_segment(pd_phys, image, offset, vaddr, filesz, memsz, flags)?;
    }

    Ok(entry)
}

struct ElfHeader {
    entry: usize,
    phoff: usize,
    phentsize: usize,
    phnum: usize,
}

fn parse_header(image: &[u8]) -> Result<ElfHeader, i32> {
    if image.len() < ELF32_HEADER_SIZE || image[..4] != [0x7F, b'E', b'L', b'F'] {
        return Err(ENOEXEC);
    }
    let class = image[4];
    let data = image[5];
    let e_type = u16_at(image, 16).ok_or(ENOEXEC)?;
    let e_machine = u16_at(image, 18).ok_or(ENOEXEC)?;
    if class != 1 || data != 1 || e_type != 2 || e_machine != 3 {
        return Err(ENOEXEC);
    }
    let phentsize = u16_at(image, 42).ok_or(ENOEXEC)? as usize;
    let phnum = u16_at(image, 44).ok_or(ENOEXEC)? as usize;
    if phentsize < PROGRAM_HEADER_SIZE {
        return Err(ENOEXEC);
    }
    Ok(ElfHeader {
        entry: u32_at(image, 24).ok_or(ENOEXEC)? as usize,
        phoff: u32_at(image, 28).ok_or(ENOEXEC)? as usize,
        phentsize,
        phnum,
    })
}

fn load_segment(
    pd_phys: usize,
    image: &[u8],
    offset: usize,
    vaddr: usize,
    filesz: usize,
    memsz: usize,
    flags: u32,
) -> Result<(), i32> {
    let writable = (flags & PF_W) != 0;
    let map_flags = PAGE_USER | if writable { PAGE_WRITABLE } else { 0 };
    let start = vaddr & !(PAGE_SIZE - 1);
    let end = (vaddr + memsz + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

    for page in (start..end).step_by(PAGE_SIZE) {
        if vmm::get_phys_addr_in(pd_phys, page).is_some() {
            return Err(ENOEXEC);
        }
        let frame = match crate::pmm::alloc_frame() {
            Some(f) => f,
            None => return Err(-12),
        };
        unsafe {
            core::ptr::write_bytes(frame as *mut u8, 0, PAGE_SIZE);
        }
        if vmm::map_page_in(pd_phys, page, frame, map_flags).is_err() {
            crate::pmm::free_frame(frame);
            return Err(-12);
        }

        let lo = vaddr.max(page);
        let hi = (vaddr + filesz).min(page + PAGE_SIZE);
        if hi > lo {
            let src = offset + (lo - vaddr);
            let len = hi - lo;
            if image.get(src..src + len).is_none() {
                return Err(ENOEXEC);
            }
            unsafe {
                core::ptr::copy_nonoverlapping(
                    image[src..src + len].as_ptr(),
                    (frame + (lo - page)) as *mut u8,
                    len,
                );
            }
        }
    }
    Ok(())
}
