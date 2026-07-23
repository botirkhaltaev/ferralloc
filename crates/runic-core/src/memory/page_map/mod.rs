use core::{
    mem::size_of,
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

use spin::Mutex;

use crate::{
    heap::{Extent, Run},
    memory::{Mapping, OsMemory, PAGE_SIZE},
};

mod entry;
mod page;
mod table;

#[cfg(test)]
mod tests;

use entry::MapEntry;
use page::{Page, PageRange};
use table::L1Table;

const PAGE_SHIFT: usize = 12;
const L2_BITS: usize = 12;
const L2_ENTRIES: usize = 1 << L2_BITS;
const L1_ENTRIES: usize = 1 << (48 - PAGE_SHIFT - L2_BITS);
const ADDRESSABLE_PAGES: usize = L1_ENTRIES * L2_ENTRIES;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PageMapError {
    InvalidRange,
    MetadataAllocFailed,
    Overlap,
    UnexpectedEntry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PageOwner {
    // Pointers must refer to live arena entries until their page-map range is removed.
    Run(NonNull<Run>),
    // Pointers must refer to live arena entries until their page-map range is removed.
    Extent(NonNull<Extent>),
}

pub(crate) struct PageMap {
    l1: AtomicPtr<L1Table>,
    l1_mapping: Mutex<Option<Mapping>>,
}

impl PageMap {
    pub(crate) const fn new() -> Self {
        Self {
            l1: AtomicPtr::new(core::ptr::null_mut()),
            l1_mapping: Mutex::new(None),
        }
    }

    pub(crate) fn get(&self, ptr: NonNull<u8>) -> Option<PageOwner> {
        let (l1_index, l2_index) = Page::containing(ptr).indexes()?;

        self.l1()?.page_entry(l1_index, l2_index)?.owner()
    }

    pub(crate) fn publish_run(
        &self,
        mapping: &Mapping,
        run: NonNull<Run>,
    ) -> Result<(), PageMapError> {
        let range = PageRange::from_mapping(mapping).ok_or(PageMapError::InvalidRange)?;
        self.insert(range, PageOwner::Run(run))
    }

    pub(crate) fn publish_extent(
        &self,
        mapping: &Mapping,
        extent: NonNull<Extent>,
    ) -> Result<(), PageMapError> {
        let range = PageRange::from_mapping(mapping).ok_or(PageMapError::InvalidRange)?;
        self.insert(range, PageOwner::Extent(extent))
    }

    pub(crate) fn unpublish_extent(
        &self,
        mapping: &Mapping,
        extent: NonNull<Extent>,
    ) -> Result<(), PageMapError> {
        let range = PageRange::from_mapping(mapping).ok_or(PageMapError::InvalidRange)?;
        self.remove(range, PageOwner::Extent(extent))
    }

    fn insert(&self, range: PageRange, entry: PageOwner) -> Result<(), PageMapError> {
        let mut l1_mapping = self.l1_mapping.lock();
        let occupied = MapEntry::from_owner(entry).ok_or(PageMapError::InvalidRange)?;

        self.validate_insert(range)?;
        self.prepare_insert(&mut l1_mapping, range)?;

        let result = if let Some(l1) = self.l1() {
            let mut result = Ok(());

            for segment in range.segments() {
                if let Err(error) = l1.entry(segment.l1)?.assign(segment.l2, occupied) {
                    result = Err(error);
                    break;
                }
            }

            result
        } else {
            Err(PageMapError::MetadataAllocFailed)
        };

        if let Err(error) = result {
            self.rollback_insert(range, occupied);

            return Err(error);
        }

        Ok(())
    }

    fn remove(&self, range: PageRange, expected: PageOwner) -> Result<(), PageMapError> {
        let _l1_mapping = self.l1_mapping.lock();
        self.validate_remove(range, expected)?;

        let l1 = self.l1().ok_or(PageMapError::UnexpectedEntry)?;
        for segment in range.segments() {
            l1.entry(segment.l1)?.clear_segment(segment.l2)?;
        }

        Ok(())
    }

    fn rollback_insert(&self, range: PageRange, entry: MapEntry) {
        let Some(l1) = self.l1() else {
            return;
        };

        for segment in range.segments() {
            if l1
                .entry(segment.l1)
                .and_then(|entry_slot| entry_slot.owns_segment(segment.l2, entry))
                != Ok(true)
            {
                continue;
            }

            let _ = l1
                .entry(segment.l1)
                .and_then(|entry_slot| entry_slot.clear_segment(segment.l2));
        }
    }

    fn l1(&self) -> Option<&L1Table> {
        let l1 = NonNull::new(self.l1.load(Ordering::Acquire))?;

        // SAFETY: `l1` points at the anonymous mmap owned by `l1_mapping` until PageMap drop.
        // That mapping is zero-filled, so the `L1Table` / `L1Entry` / nested `L2Mapping` bit
        // pattern is valid before first install (null `AtomicPtr`, `Option::None`, zero counts).
        Some(unsafe { l1.as_ref() })
    }

    fn l1_or_init(&self, l1_mapping: &mut Option<Mapping>) -> Result<&L1Table, PageMapError> {
        if self.l1.load(Ordering::Acquire).is_null() {
            let mapping =
                OsMemory::map(size_of::<L1Table>()).ok_or(PageMapError::MetadataAllocFailed)?;
            let ptr = mapping.base().cast::<L1Table>().as_ptr();
            // Anonymous mmap is zero-filled: a valid empty `L1Table` before any L2 install.
            // Store ownership in `l1_mapping` before publishing the atomic pointer for `get`.
            *l1_mapping = Some(mapping);
            self.l1.store(ptr, Ordering::Release);
        }

        self.l1().ok_or(PageMapError::MetadataAllocFailed)
    }

    fn validate_insert(&self, range: PageRange) -> Result<(), PageMapError> {
        let Some(l1) = self.l1() else {
            return Ok(());
        };

        let empty = MapEntry::empty();
        for segment in range.segments() {
            if !l1.entry(segment.l1)?.owns_segment(segment.l2, empty)? {
                return Err(PageMapError::Overlap);
            }
        }

        Ok(())
    }

    fn validate_remove(&self, range: PageRange, expected: PageOwner) -> Result<(), PageMapError> {
        let expected = MapEntry::from_owner(expected).ok_or(PageMapError::InvalidRange)?;

        let Some(l1) = self.l1() else {
            return Err(PageMapError::UnexpectedEntry);
        };

        for segment in range.segments() {
            if !l1.entry(segment.l1)?.owns_segment(segment.l2, expected)? {
                return Err(PageMapError::UnexpectedEntry);
            }
        }

        Ok(())
    }

    fn prepare_insert(
        &self,
        l1_mapping: &mut Option<Mapping>,
        range: PageRange,
    ) -> Result<(), PageMapError> {
        let l1 = self.l1_or_init(l1_mapping)?;

        for segment in range.segments() {
            l1.ensure_l2_table(segment.l1)?;
        }

        Ok(())
    }
}

impl Drop for PageMap {
    fn drop(&mut self) {
        let Some(mut l1_ptr) = NonNull::new(*self.l1.get_mut()) else {
            return;
        };
        *self.l1.get_mut() = core::ptr::null_mut();

        // SAFETY: PageMap drop has unique access to the L1 table.
        let l1 = unsafe { l1_ptr.as_mut() };

        for entry in &mut l1.entries {
            entry.drop_l2_mapping();
        }

        let _ = self.l1_mapping.get_mut().take();
    }
}

const _: () = assert!(
    PAGE_SIZE == 1 << PAGE_SHIFT,
    "PAGE_SHIFT must match PAGE_SIZE"
);
