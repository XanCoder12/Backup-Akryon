# Virtual Memory Manager (VMM) & x86 Paging

The Virtual Memory Manager (`rust/src/vmm.rs`) implements standard two-level x86 page translation, memory protection, linear framebuffer mapping, and demand paging triggered via CPU Exception 14 (Page Fault).

## Two-Level Page Translation Architecture

x86 32-bit hardware paging maps a 32-bit linear virtual address into a 32-bit physical address through a Page Directory and Page Tables:

```
Virtual Address (32-bit):
+--------------------+--------------------+--------------------+
| Directory (10 bit) | Table (10 bit)     | Offset (12 bit)    |
| Bits 31..22        | Bits 21..12        | Bits 11..0         |
+--------------------+--------------------+--------------------+
         |                    |                    |
         v                    v                    v
  [Page Directory] ---> [Page Table] -------> [Physical 4KB Frame]
  (CR3 Register)
```

## Physical Placement in Extended DRAM

All core paging structures reside in safe extended physical memory above 1 MB:
- **`0x00100000` (`PAGE_DIR_PHYS`)**: Page Directory (4 KB, 1,024 entries).
- **`0x00101000` (`LFB_TABLE_PHYS`)**: Linear Framebuffer Page Table (4 KB).
- **`0x00110000` (`IDENT_TABLES_PHYS`)**: 16 Identity Page Tables (64 KB total).

## Identity Mapping (0 to 64 MB)

To ensure the kernel code, HAL data, stacks, heap, and memory-mapped BIOS regions execute transparently without address translation overhead:
- 16 Page Tables are populated to identity map `0x00000000` through `0x03FFFFFF` (64 MB).
- Every entry has flags: `PAGE_PRESENT | PAGE_WRITABLE` (`0x03`).
- Page Directory entries `0` through `15` point directly to these identity tables.

## Framebuffer Memory-Mapped I/O

The VBE Linear Framebuffer is mapped to ensure video memory writes bypass cache buffering:
- Page Directory index computed as `(lfb_phys >> 22) & 0x3FF`.
- Entries flagged with `PAGE_PRESENT | PAGE_WRITABLE | PAGE_NOCACHE` (`0x13`).

## Demand Paging Subsystem

Nyxara supports dynamic demand paging within a reserved 16 MB virtual address window:
- **Range**: `0xC0000000` to `0xC1000000`.
- When an instruction reads or writes to this range, the page table initially marks the page as not present (`P = 0`).
- The CPU immediately triggers an **ISR 14 Page Fault**.

### Page Fault Handler (`isr14_page_fault`)

```rust
extern "C" fn isr14_page_fault(regs: &mut Registers) {
    let fault_addr = unsafe { read_cr2() };
    let err_code = regs.err_code;

    // Check if fault occurred within Demand Paging region
    if fault_addr >= DEMAND_PAGING_START && fault_addr < DEMAND_PAGING_END {
        // 1. Allocate a physical frame from PMM
        let frame_phys = match pmm::alloc_frame() {
            Some(addr) => addr,
            None => panic!("Out of physical memory during demand paging!"),
        };

        // 2. Map the frame into the Page Directory / Page Table
        unsafe {
            map_demand_page(fault_addr, frame_phys);
            invalidate_tlb(fault_addr); // invlpg instruction
            DEMAND_PAGE_FAULTS += 1;
        }

        // Return from interrupt: CPU restarts the faulting instruction seamlessly
        return;
    }

    // Unhandled crash outside demand region
    panic!("Fatal Page Fault at {:#010X} (EIP: {:#010X})", fault_addr, regs.eip);
}
```

## Activating Paging

During `vmm::init()`:
1. Load Page Directory address into `CR3`:
   ```rust
   core::arch::asm!("mov cr3, {0}", in(reg) PAGE_DIR_PHYS);
   ```
2. Enable Write-Protect (`WP`, bit 16) and Paging (`PG`, bit 31) in `CR0`:
   ```rust
   let mut cr0: usize;
   core::arch::asm!("mov {0}, cr0", out(reg) cr0);
   cr0 |= (1 << 31) | (1 << 16);
   core::arch::asm!("mov cr0, {0}", in(reg) cr0);
   ```

## Automated Verification (`test_vmm`)

At boot time, `vmm::test_vmm()` verifies demand paging by writing known magic patterns (`0xDEADBEEF`, `0xCAFEBABE`) to unmapped addresses `0xC0000000` and `0xC0001000`. It confirms that the Page Fault handler allocates frames on-the-fly and resumes execution without crashing.
