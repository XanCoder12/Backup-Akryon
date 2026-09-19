# Physical Memory Manager (PMM)

The Physical Memory Manager (`rust/src/pmm.rs`) manages physical DRAM page frames using a bitmap allocator. Every 4 KB physical page frame corresponds to a single bit in a statically allocated bitmap array.

## Core Specifications

- **Frame (Page) Size**: 4,096 bytes (4 KB).
- **Supported Physical Memory**: Up to 128 MB (`128 * 1024 * 1024` bytes).
- **Total Manageable Frames**: 32,768 frames.
- **Bitmap Size**: 1,024 32-bit words (`32,768 / 32 = 1,024` entries of `u32`, totaling 4 KB).
- **Bit Semantics**:
  - `1` = Frame is **Allocated** or **Reserved** (unavailable).
  - `0` = Frame is **Free** (available for allocation).

## Bitmap Data Structures

```rust
pub const PAGE_SIZE: usize = 4096;
const MAX_PHYSICAL_MEM: usize = 128 * 1024 * 1024;
const TOTAL_PAGES: usize = MAX_PHYSICAL_MEM / PAGE_SIZE;
const BITMAP_ENTRIES: usize = TOTAL_PAGES / 32;

static mut BITMAP: [u32; BITMAP_ENTRIES] = [0xFFFFFFFF; BITMAP_ENTRIES];
static mut TOTAL_FRAMES: usize = 0;
static mut USED_FRAMES: usize = 0;
```

## Initialization Sequence (`pmm::init`)

When `pmm::init(mem_size, kernel_start, kernel_end)` is called during system boot:

1. **Conservative Default**: All frames across the entire bitmap are initially marked as used (`0xFFFFFFFF`).
2. **Release Free Memory**: All frames from `kernel_end` up to `mem_size` are marked free (`free_frame_raw`).
3. **Protect Kernel Code & Data**: Frames spanning `kernel_start..kernel_end` are explicitly reserved.
4. **Protect Lower 1 MB**: Pages 0 through 255 (`0x00000000..0x00100000`) are permanently marked as used to protect the Real Mode IVT, BIOS Data Area (BDA), MBR scratch spaces, and VGA buffers.
5. **Protect Kernel Stack & Extended Tables**: Extended ranges `0x00100000..0x00140000` (holding the Page Directory, Page Tables, and Kernel Stack) are reserved via `reserve_frame()`.

## Allocation Algorithm (`alloc_frame`)

The allocator searches for the first available frame using fast bitwise operations:

```rust
pub fn alloc_frame() -> Option<usize> {
    unsafe {
        let max_idx = (TOTAL_FRAMES + 31) / 32;
        for i in 0..max_idx {
            if BITMAP[i] != 0xFFFFFFFF {
                let bit = BITMAP[i].trailing_ones() as usize;
                if bit < 32 {
                    let frame = i * 32 + bit;
                    if frame < TOTAL_FRAMES {
                        BITMAP[i] |= 1 << bit;
                        USED_FRAMES += 1;
                        return Some(frame * PAGE_SIZE);
                    }
                }
            }
        }
        None // Out of physical memory
    }
}
```

- **`trailing_ones()`**: Uses CPU instruction `tzcnt` / bit scan to find the lowest 0-bit in `O(1)` time per 32-bit word.
- Returns physical address `frame * 4096`.

## Deallocation (`free_frame`)

```rust
pub fn free_frame(addr: usize) {
    unsafe {
        free_frame_raw(addr);
    }
}
```
Clears the bit at `(addr / 4096) % 32` within word `(addr / 4096) / 32` and decrements `USED_FRAMES`.

## Memory Inspection APIs

- `pmm::total_memory()`: Total manageable physical bytes.
- `pmm::used_memory()`: Number of allocated bytes (`USED_FRAMES * 4096`).
- `pmm::free_memory()`: Number of unallocated bytes (`(TOTAL_FRAMES - USED_FRAMES) * 4096`).
- Integrated into the shell via the `free` and `meminfo` commands.
