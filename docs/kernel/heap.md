# Kernel Heap Allocator

The Nyxara Kernel Heap (`rust/src/heap.rs`) implements the standard Rust `core::alloc::GlobalAlloc` trait. This bridges low-level memory with the high-level `extern crate alloc`, unlocking dynamic collections such as `Vec`, `String`, `Box`, and the `format!` macro within the `#![no_std]` environment.

## Architecture

The allocator is an intrusive first-fit free list: every block stays in the list for its whole life and is marked allocated or free. Allocation splits the first fitting free block; deallocation merges address-adjacent free blocks. `alloc`/`dealloc` run with interrupts disabled (`pushfd`/`cli`, flags restored on exit) so timer or keyboard IRQs cannot re-enter the allocator mid-mutation.

### Block Layout

```rust
struct BlockHeader {
    size: usize,             // Usable payload bytes (excluding header)
    free: u32,               // 1 = free, 0 = in-use
    next: *mut BlockHeader,  // Next block in the address-ordered list
    magic: u32,              // 0x4E595241 ("NYRA"), detects header corruption
}
```

Blocks are 16-byte aligned and the payload always starts at `block + 16`, so every live pointer is 16-aligned and its header sits exactly at `ptr - 16`. `dealloc` therefore computes the header directly and validates the magic before freeing; a pointer whose header was overwritten is logged and leaked instead of being guessed back into the pool. Layouts with alignment above 16 are not supported.

A failing `alloc` walks the list and logs block/free counts, free bytes, and the largest free block over serial (`[Heap] alloc(N) failed: ...`) before returning null.

## Heap Initialization

During boot in `nyxara_rust_main()`:
1. `heap_start` is calculated from the linker symbol `kernel_end`, aligned to the next 4 KB boundary.
2. A 4 MB memory region is reserved in the PMM.
3. `heap::init(heap_start, heap_size)` initializes the single free block spanning the region.
4. The `#[global_allocator]` static lives in `heap.rs` itself.

## Observability

`free` reports kernel-heap usage alongside PMM statistics:

```
Kernel Heap:
  Used  : 10 KB / 4096 KB (peak 27 KB)
```

`heap::heap_used()`, `heap::heap_peak()`, and `heap::heap_capacity()` expose the counters.

## Integration with Rust Collections

By registering:
```rust
#[global_allocator]
static ALLOCATOR: heap::KernelAllocator = heap::KernelAllocator::empty();
```
All standard Rust heap constructs are available across the kernel:
- `alloc::string::String` and `alloc::string::ToString`.
- `alloc::vec::Vec`.
- `alloc::boxed::Box`.
- String formatting: `alloc::format!("{}:{}", host, port)`.
