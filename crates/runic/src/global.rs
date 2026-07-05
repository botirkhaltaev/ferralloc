use core::alloc::{GlobalAlloc, Layout};

use runic_core::Allocator;

pub struct RunicAlloc;

unsafe impl GlobalAlloc for RunicAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { Allocator::alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { Allocator::dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, old: Layout, new_size: usize) -> *mut u8 {
        unsafe { Allocator::realloc(ptr, old, new_size) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { Allocator::alloc_zeroed(layout) }
    }
}
