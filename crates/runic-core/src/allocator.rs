use core::{alloc::Layout, cell::UnsafeCell};

use crate::heap::Heap;

static HEAP: GlobalHeap = GlobalHeap::new();

struct GlobalHeap {
    heap: UnsafeCell<Heap>,
}

// SAFETY: this is a temporary single-threaded diagnostic configuration. Any
// concurrent access to the process-global allocator is a data race and UB.
unsafe impl Sync for GlobalHeap {}

impl GlobalHeap {
    const fn new() -> Self {
        Self {
            heap: UnsafeCell::new(Heap::new()),
        }
    }

    fn heap_ptr(&self) -> *mut Heap {
        self.heap.get()
    }
}

#[non_exhaustive]
pub struct Allocator;

impl Allocator {
    /// Allocates memory for `layout` using the process-global Runic state.
    ///
    /// # Safety
    ///
    /// The returned pointer is raw, uninitialized memory. The caller must use it
    /// only according to `layout`, avoid out-of-bounds access, and eventually
    /// pass the same pointer and a compatible layout back to this allocator.
    pub unsafe fn alloc(layout: Layout) -> *mut u8 {
        // SAFETY: this lockless diagnostic allocator is valid only for single-threaded execution.
        let heap = unsafe { &mut *HEAP.heap_ptr() };
        heap.alloc(layout)
    }

    /// Deallocates memory previously returned by this allocator.
    ///
    /// # Safety
    ///
    /// `ptr` must be null or a pointer previously returned by this allocator
    /// for `layout`. Passing an unknown pointer, an interior pointer, or an
    /// incompatible layout violates the allocator contract and may abort.
    pub unsafe fn dealloc(ptr: *mut u8, layout: Layout) {
        // SAFETY: this lockless diagnostic allocator is valid only for single-threaded execution.
        let heap = unsafe { &mut *HEAP.heap_ptr() };
        if heap.dealloc(ptr, layout).is_err() {
            Self::abort();
        }
    }

    /// Changes the size of an allocation using allocate-copy-free semantics.
    ///
    /// # Safety
    ///
    /// `ptr` must be null or a pointer previously returned by this allocator
    /// for `old`. If a non-null pointer is supplied, no other live reference may
    /// be used to access the old allocation after successful reallocation.
    pub unsafe fn realloc(ptr: *mut u8, old: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: this lockless diagnostic allocator is valid only for single-threaded execution.
        let heap = unsafe { &mut *HEAP.heap_ptr() };
        heap.realloc(ptr, old, new_size)
            .unwrap_or_else(|_| Self::abort())
    }

    /// Allocates zero-initialized memory for `layout`.
    ///
    /// # Safety
    ///
    /// The returned pointer is raw memory. The caller must use it only according
    /// to `layout` and eventually pass it back to this allocator with a
    /// compatible layout.
    pub unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
        // SAFETY: this lockless diagnostic allocator is valid only for single-threaded execution.
        let heap = unsafe { &mut *HEAP.heap_ptr() };
        heap.alloc_zeroed(layout)
    }

    #[cold]
    #[inline(never)]
    fn abort() -> ! {
        // SAFETY: abort terminates the process and does not unwind across allocator boundaries.
        unsafe { libc::abort() }
    }
}
