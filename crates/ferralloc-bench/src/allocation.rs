use std::{alloc::Layout, ptr::NonNull};

use crate::allocator_target::AllocatorTarget;

pub struct AllocationRecord {
    target: AllocatorTarget,
    ptr: NonNull<u8>,
    layout: Layout,
    id: u64,
}

impl AllocationRecord {
    pub fn new(target: AllocatorTarget, layout: Layout, id: u64) -> Self {
        let ptr = target.alloc(layout);
        assert_eq!(ptr.as_ptr() as usize % layout.align(), 0);
        let record = Self {
            target,
            ptr,
            layout,
            id,
        };
        record.write_markers();
        record
    }

    pub fn zeroed(target: AllocatorTarget, layout: Layout, id: u64) -> Self {
        let ptr = target.alloc_zeroed(layout);
        assert_eq!(ptr.as_ptr() as usize % layout.align(), 0);
        for index in marker_indices(layout.size()) {
            assert_eq!(unsafe { ptr.as_ptr().add(index).read() }, 0);
        }
        let record = Self {
            target,
            ptr,
            layout,
            id,
        };
        record.write_markers();
        record
    }

    pub fn ptr(&self) -> NonNull<u8> {
        self.ptr
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }

    pub fn write_pattern(&self) {
        for index in 0..self.layout.size() {
            unsafe { self.ptr.as_ptr().add(index).write(self.byte_at(index)) };
        }
    }

    pub fn write_markers(&self) {
        for index in marker_indices(self.layout.size()) {
            unsafe { self.ptr.as_ptr().add(index).write(self.byte_at(index)) };
        }
    }

    pub fn check_pattern(&self) {
        self.check_markers();
    }

    pub fn check_markers(&self) {
        for index in marker_indices(self.layout.size()) {
            let byte = unsafe { self.ptr.as_ptr().add(index).read() };
            assert_eq!(byte, self.byte_at(index));
        }
    }

    pub fn check_prefix(&self, ptr: NonNull<u8>, len: usize) {
        for index in 0..len {
            let byte = unsafe { ptr.as_ptr().add(index).read() };
            assert_eq!(byte, self.byte_at(index));
        }
    }

    pub fn check_prefix_markers(&self, ptr: NonNull<u8>, len: usize) {
        for index in marker_indices(self.layout.size())
            .into_iter()
            .filter(|&index| index < len)
        {
            let byte = unsafe { ptr.as_ptr().add(index).read() };
            assert_eq!(byte, self.byte_at(index));
        }
    }

    pub fn realloc(&mut self, new_size: usize) {
        self.check_pattern();
        let old = self.layout;
        let new_ptr = self.target.realloc(self.ptr, old, new_size);
        assert_eq!(new_ptr.as_ptr() as usize % old.align(), 0);
        self.check_prefix_markers(new_ptr, old.size().min(new_size));
        self.ptr = new_ptr;
        self.layout = Layout::from_size_align(new_size, old.align()).unwrap();
        self.write_markers();
    }

    pub fn dealloc(self) {
        let target = self.target;
        let ptr = self.ptr;
        let layout = self.layout;
        std::mem::forget(self);
        target.dealloc(ptr, layout);
    }

    fn byte_at(&self, index: usize) -> u8 {
        self.id
            .wrapping_mul(131)
            .wrapping_add(index as u64)
            .wrapping_add(self.layout.size() as u64) as u8
    }
}

fn marker_indices(size: usize) -> [usize; 3] {
    debug_assert!(size > 0);
    [0, size / 2, size - 1]
}

impl Drop for AllocationRecord {
    fn drop(&mut self) {
        self.target.dealloc(self.ptr, self.layout);
    }
}
