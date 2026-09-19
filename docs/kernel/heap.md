# Kernel Heap Allocator

The Nyxara Kernel Heap (`rust/src/heap.rs`) implements the standard Rust `core::alloc::GlobalAlloc` trait. This bridges low-level memory with the high-level `extern crate alloc`, unlocking dynamic collections such as `Vec`, `String`, `Box`, and the `format!` macro within the `#![no_std]` environment.

## Architecture

The allocator uses an intrusive linked-list first-fit algorithm with block splitting on allocation and adjacent block coalescing on deallocation.

### Block Header Structure
Every allocated or free memory chunk is prefixed by an intrusive header:

```rust
struct BlockHeader {
    size: usize,             // Size of usable payload (excluding header)
    free: bool,              // Allocation status (true = free, false = in-use)
    next: *mut BlockHeader,  // Pointer to next block in heap pool
}
```

## Heap Initialization

During boot in `nyxara_rust_main()`:
1. `heap_start` is calculated from the linker symbol `kernel_end`, aligned to the next 4 KB boundary.
2. A 4 MB memory region is reserved in the PMM (`0x400000` bytes).
3. The allocator is initialized:
   ```rust
   let heap_size = 4 * 1024 * 1024; // 4 MB
   let heap_start = (k_end + 4095) & !4095;
   unsafe {
       ALLOCATOR.init(heap_start, heap_size);
   }
   ```
4. A single large `BlockHeader` spanning the available 4 MB space is placed at `heap_start`.

## Allocation Algorithm (`alloc`)

When `alloc(layout: Layout)` is invoked:
1. Computes the required byte size, rounded up to an 8-byte boundary.
2. Traverses the linked list from `head` looking for the first `free == true` block with sufficient capacity:
   ```rust
   let total_needed = adjustment + needed_size;
   if header.size >= total_needed {
       // Check if block can be split
       let remaining = header.size - total_needed;
       if remaining >= core::mem::size_of::<BlockHeader>() + 16 {
           // Split block into allocated front and new free block
           let next_block = (data_ptr as usize + needed_size) as *mut BlockHeader;
           (*next_block).size = remaining - core::mem::size_of::<BlockHeader>();
           (*next_block).free = true;
           (*next_block).next = header.next;
           header.next = next_block;
           header.size = total_needed;
       }
       header.free = false;
       return data_ptr;
   }
   ```
3. If no block satisfies the request, returns `ptr::null_mut()`.

## Deallocation & Coalescing (`dealloc`)

When memory is released:
1. Finds the `BlockHeader` immediately preceding the pointer:
   ```rust
   let header = (ptr as usize - core::mem::size_of::<BlockHeader>()) as *mut BlockHeader;
   (*header).free = true;
   ```
2. **Coalescing**: Iterates through the list and merges adjacent contiguous free blocks into a single larger block, preventing external memory fragmentation.

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
