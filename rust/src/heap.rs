use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;

const HEADER_MAGIC: u32 = 0x4E595241; // "NYRA"
const BLOCK_ALIGN: usize = 16;
// Data starts at block + BLOCK_ALIGN, so every live pointer is 16-aligned
// and its header sits exactly at ptr - BLOCK_ALIGN. Layouts with align > 16
// are not supported.
const HEADER_SIZE: usize = 16;
const MIN_SPLIT: usize = HEADER_SIZE + 16;

struct BlockHeader {
    size: usize,
    free: u32,
    next: *mut BlockHeader,
    magic: u32,
}

struct AllocatorInner {
    head: *mut BlockHeader,
    heap_start: usize,
    heap_end: usize,
}

pub struct KernelAllocator {
    inner: UnsafeCell<AllocatorInner>,
}

#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator::empty();

unsafe impl Send for KernelAllocator {}
unsafe impl Sync for KernelAllocator {}

impl KernelAllocator {
    pub const fn empty() -> Self {
        Self {
            inner: UnsafeCell::new(AllocatorInner {
                head: ptr::null_mut(),
                heap_start: 0,
                heap_end: 0,
            }),
        }
    }

    pub unsafe fn init(&self, start: usize, size: usize) {
        let inner = &mut *self.inner.get();
        let base = (start + BLOCK_ALIGN - 1) & !(BLOCK_ALIGN - 1);
        inner.heap_start = base;
        inner.heap_end = start + size;
        inner.head = base as *mut BlockHeader;

        let available = (inner.heap_end - base).saturating_sub(HEADER_SIZE) & !(BLOCK_ALIGN - 1);
        (*inner.head).size = available;
        (*inner.head).free = 1;
        (*inner.head).magic = HEADER_MAGIC;
        (*inner.head).next = ptr::null_mut();
    }
}

static mut USED_BYTES: usize = 0;
static mut PEAK_BYTES: usize = 0;

pub fn heap_used() -> usize {
    unsafe { USED_BYTES }
}

pub fn heap_peak() -> usize {
    unsafe { PEAK_BYTES }
}

pub fn heap_capacity() -> usize {
    unsafe {
        let inner = &*ALLOCATOR.inner.get();
        inner.heap_end - inner.heap_start
    }
}

pub fn init(start: usize, size: usize) {
    unsafe {
        ALLOCATOR.init(start, size);
    }
}

#[inline]
fn irq_save() -> u32 {
    let flags: u32;
    unsafe {
        core::arch::asm!(
            "pushfd",
            "pop {0}",
            "cli",
            out(reg) flags,
            options(nomem, preserves_flags)
        );
    }
    flags
}

#[inline]
fn irq_restore(flags: u32) {
    unsafe {
        core::arch::asm!("push {0}", "popfd", in(reg) flags, options(nomem));
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = alloc_block(layout);
        if !ptr.is_null() {
            USED_BYTES += layout.size();
            if USED_BYTES > PEAK_BYTES {
                PEAK_BYTES = USED_BYTES;
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }
        dealloc_block(ptr);
        USED_BYTES = USED_BYTES.saturating_sub(layout.size());
    }
}

unsafe fn alloc_block(layout: Layout) -> *mut u8 {
    let inner = &mut *ALLOCATOR.inner.get();
    let needed = ((layout.size() + BLOCK_ALIGN - 1) & !(BLOCK_ALIGN - 1)).max(BLOCK_ALIGN);

    let mut curr = inner.head;
    while !curr.is_null() {
        if header_ok(curr) && (*curr).free == 1 && (*curr).size >= needed {
            // The leftover block's own header eats into the surplus data.
            if (*curr).size >= needed + MIN_SPLIT {
                let split = (curr as usize + HEADER_SIZE + needed) as *mut BlockHeader;
                (*split).size = (*curr).size - needed - HEADER_SIZE;
                (*split).free = 1;
                (*split).magic = HEADER_MAGIC;
                (*split).next = (*curr).next;
                (*curr).next = split;
                (*curr).size = needed;
            }
            (*curr).free = 0;
            return (curr as usize + HEADER_SIZE) as *mut u8;
        }
        curr = (*curr).next;
    }

    dump_free_list(layout.size());
    ptr::null_mut()
}

unsafe fn dealloc_block(ptr: *mut u8) {
    let header = (ptr as usize - HEADER_SIZE) as *mut BlockHeader;
    if !header_ok(header) {
        crate::logln!(
            "[Heap] dealloc of 0x{:08X} has no valid header; block leaked.",
            ptr as usize
        );
        return;
    }
    (*header).free = 1;
    merge_free();
}

unsafe fn merge_free() {
    let inner = &mut *ALLOCATOR.inner.get();
    let mut curr = inner.head;
    while !curr.is_null() {
        let next = (*curr).next;
        let expected = (curr as usize + HEADER_SIZE + (*curr).size) as *mut BlockHeader;
        if next == expected && header_ok(next) && (*next).free == 1 && (*curr).free == 1 {
            (*curr).size += HEADER_SIZE + (*next).size;
            (*curr).next = (*next).next;
            continue;
        }
        curr = next;
    }
}

#[inline]
unsafe fn header_ok(header: *mut BlockHeader) -> bool {
    !header.is_null() && (*header).magic == HEADER_MAGIC
}

unsafe fn dump_free_list(requested: usize) {
    let inner = &*ALLOCATOR.inner.get();
    let mut curr = inner.head;
    let (mut blocks, mut free_blocks, mut free_bytes, mut largest) =
        (0usize, 0usize, 0usize, 0usize);
    while !curr.is_null() {
        blocks += 1;
        if header_ok(curr) {
            free_blocks += 1;
            free_bytes += (*curr).size;
            if (*curr).size > largest {
                largest = (*curr).size;
            }
        }
        curr = (*curr).next;
    }
    crate::logln!(
        "[Heap] alloc({}) failed: blocks={} free_blocks={} free_bytes={} largest={}",
        requested,
        blocks,
        free_blocks,
        free_bytes,
        largest
    );
}
