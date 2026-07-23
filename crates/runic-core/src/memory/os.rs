use core::{num::NonZeroUsize, ptr::NonNull};

use crate::memory::AddressRange;

pub(crate) const PAGE_SIZE: usize = 4096;

/// Sole owner of one live anonymous mmap region.
///
/// Constructed only by [`OsMemory::map`]. `Drop` munmaps the region. Length is
/// always nonzero and a multiple of [`PAGE_SIZE`]; base is always page-aligned.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Mapping {
    base: NonNull<u8>,
    len: NonZeroUsize,
}

impl Mapping {
    /// Private: every `Mapping` must describe a live mmap region owned uniquely
    /// by that `Mapping`, so construction is confined to `OsMemory::map`.
    fn new(base: NonNull<u8>, len: NonZeroUsize) -> Self {
        debug_assert!(base.as_ptr().addr().is_multiple_of(PAGE_SIZE));
        debug_assert!(len.get().is_multiple_of(PAGE_SIZE));
        Self { base, len }
    }

    pub(crate) const fn base(&self) -> NonNull<u8> {
        self.base
    }

    pub(crate) const fn len(&self) -> NonZeroUsize {
        self.len
    }

    pub(crate) const fn range(&self) -> AddressRange {
        AddressRange::new(self.base, self.len.get())
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        // SAFETY: Mapping owns an mmap allocation returned by OsMemory::map.
        unsafe { libc::munmap(self.base.as_ptr().cast(), self.len.get()) };
    }
}

// SAFETY: Mapping owns a process-private mmap region. Moving ownership to another
// thread does not permit concurrent mutation of allocator metadata.
unsafe impl Send for Mapping {}

pub(crate) struct OsMemory;

impl OsMemory {
    pub(crate) const fn page_size() -> usize {
        PAGE_SIZE
    }

    pub(crate) fn map(len: usize) -> Option<Mapping> {
        if len == 0 {
            return None;
        }

        let rounded_len = Self::round_to_page(len)?;
        let rounded_len = NonZeroUsize::new(rounded_len)?;
        // SAFETY: mmap is called with a null hint, anonymous private mapping, and a page-rounded length.
        let ptr = unsafe {
            libc::mmap(
                core::ptr::null_mut(),
                rounded_len.get(),
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            return None;
        }

        NonNull::new(ptr.cast::<u8>()).map(|base| Mapping::new(base, rounded_len))
    }

    pub(crate) fn round_to_page(len: usize) -> Option<usize> {
        if len == 0 {
            return None;
        }

        let mask = PAGE_SIZE - 1;
        len.checked_add(mask).map(|value| value & !mask)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_memory_round_to_page_keeps_page_sized_value() {
        assert_eq!(OsMemory::round_to_page(PAGE_SIZE), Some(PAGE_SIZE));
    }

    #[test]
    fn os_memory_round_to_page_rounds_up() {
        assert_eq!(OsMemory::round_to_page(PAGE_SIZE + 1), Some(PAGE_SIZE * 2));
    }

    #[test]
    fn os_memory_round_to_page_detects_overflow() {
        assert_eq!(OsMemory::round_to_page(usize::MAX), None);
    }

    #[test]
    fn os_memory_round_to_page_rejects_zero() {
        assert_eq!(OsMemory::round_to_page(0), None);
    }

    #[test]
    fn os_memory_map_rejects_zero() {
        assert!(OsMemory::map(0).is_none());
    }

    #[test]
    fn os_memory_map_returns_page_aligned_mapping() {
        let mapping = OsMemory::map(1).unwrap();

        assert_eq!(mapping.base().as_ptr() as usize % PAGE_SIZE, 0);
        assert_eq!(mapping.len().get(), PAGE_SIZE);

        drop(mapping);
    }

    #[test]
    fn os_memory_mapping_is_writable() {
        let mapping = OsMemory::map(PAGE_SIZE).unwrap();

        unsafe {
            mapping.base().as_ptr().write(0xab);
            mapping.base().as_ptr().add(PAGE_SIZE - 1).write(0xcd);
            assert_eq!(mapping.base().as_ptr().read(), 0xab);
            assert_eq!(mapping.base().as_ptr().add(PAGE_SIZE - 1).read(), 0xcd);
        }
    }
}
