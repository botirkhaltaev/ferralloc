use core::{
    cell::UnsafeCell,
    mem::size_of,
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::memory::{Mapping, OsMemory};

use super::{
    L1_ENTRIES, L2_ENTRIES, PageMapError,
    entry::{AtomicMapEntry, MapEntry},
    page::{L1Index, L2Index, L2Segment},
};

#[repr(C)]
pub(super) struct L1Table {
    pub(super) entries: [L1Entry; L1_ENTRIES],
}

impl L1Table {
    pub(super) fn page_entry(&self, l1_index: L1Index, l2_index: L2Index) -> Option<MapEntry> {
        self.entries.get(l1_index.get())?.page_entry(l2_index)
    }

    pub(super) fn entry(&self, index: L1Index) -> Result<&L1Entry, PageMapError> {
        self.entries
            .get(index.get())
            .ok_or(PageMapError::InvalidRange)
    }

    /// Ensures an L2 table exists for `index`.
    ///
    /// Caller must hold `PageMap::l1_mapping`.
    pub(super) fn ensure_l2_table(&self, index: L1Index) -> Result<(), PageMapError> {
        let entry = self.entry(index)?;

        if entry.has_l2_table() {
            return Ok(());
        }

        let Some(mapping) = OsMemory::map(size_of::<L2Table>()) else {
            return Err(PageMapError::MetadataAllocFailed);
        };
        entry.install_l2_mapping(mapping);

        Ok(())
    }
}

/// L2 table mmap ownership and occupancy; mutated only while `PageMap::l1_mapping` is held.
///
/// Anonymous mmap zero-fill yields a valid value: `mapping == None`, `occupied_pages == 0`.
pub(super) struct L2Mapping {
    mapping: Option<Mapping>,
    occupied_pages: u32,
}

impl L2Mapping {
    fn install(&mut self, mapping: Mapping) {
        self.mapping = Some(mapping);
        self.occupied_pages = 0;
    }

    fn clear(&mut self) {
        self.occupied_pages = 0;
        self.mapping = None;
    }

    fn is_unoccupied(&self) -> bool {
        self.occupied_pages == 0
    }

    /// Returns the new occupied count after a successful checked add; does not store until
    /// the caller commits (after page writes succeed).
    fn try_add_pages(&self, pages: u32) -> Result<u32, PageMapError> {
        self.occupied_pages
            .checked_add(pages)
            .ok_or(PageMapError::InvalidRange)
    }

    fn try_sub_pages(&self, pages: u32) -> Result<u32, PageMapError> {
        self.occupied_pages
            .checked_sub(pages)
            .ok_or(PageMapError::UnexpectedEntry)
    }

    fn set_occupied_pages(&mut self, occupied_pages: u32) {
        self.occupied_pages = occupied_pages;
    }
}

#[repr(C)]
pub(super) struct L1Entry {
    table: AtomicPtr<L2Table>,
    l2_mapping: UnsafeCell<L2Mapping>,
}

// SAFETY: The L2 table pointer is published atomically for lock-free get. `l2_mapping` is
// only accessed while `PageMap::l1_mapping` is locked (or uniquely in PageMap drop).
unsafe impl Sync for L1Entry {}

impl L1Entry {
    pub(super) fn has_l2_table(&self) -> bool {
        !self.table.load(Ordering::Acquire).is_null()
    }

    fn l2_table(&self) -> Option<NonNull<L2Table>> {
        NonNull::new(self.table.load(Ordering::Acquire))
    }

    pub(super) fn l2_table_ref(&self) -> Option<&L2Table> {
        let table = self.l2_table()?;

        // SAFETY: l2_table returns the live L2 table pointer owned by this L1 entry.
        Some(unsafe { table.as_ref() })
    }

    fn install_l2_mapping(&self, mapping: Mapping) {
        let table = mapping.base().cast::<L2Table>().as_ptr();
        // SAFETY: caller holds `PageMap::l1_mapping`. Anonymous mmap is zero-filled, so the
        // prior `L2Mapping` / `L2Table` bit pattern was valid (`None`, null atomics, empty
        // entries). Ownership is stored before the L2 pointer is published.
        let l2_mapping = unsafe { &mut *self.l2_mapping.get() };
        l2_mapping.install(mapping);
        self.table.store(table, Ordering::Release);
    }

    pub(super) fn drop_l2_mapping(&mut self) {
        if self.table.load(Ordering::Acquire).is_null() {
            return;
        }

        self.table.store(core::ptr::null_mut(), Ordering::Release);
        self.l2_mapping.get_mut().clear();
    }

    /// Caller must hold `PageMap::l1_mapping`.
    pub(super) fn owns_segment(
        &self,
        segment: L2Segment,
        expected: MapEntry,
    ) -> Result<bool, PageMapError> {
        let Some(table) = self.l2_table_ref() else {
            return Ok(expected.is_empty());
        };

        // SAFETY: caller holds `PageMap::l1_mapping`.
        let l2_mapping = unsafe { &*self.l2_mapping.get() };
        if expected.is_empty() && l2_mapping.is_unoccupied() {
            return Ok(true);
        }

        table.owns_segment(segment, expected)
    }

    fn page_entry(&self, index: L2Index) -> Option<MapEntry> {
        self.l2_table_ref()?.get(index)
    }

    /// Assigns every page in `segment` the same page-map entry.
    ///
    /// Runs and extents share this one representation; there is no alternate
    /// encoding to fall back to, so a segment either fits directly or the
    /// insert fails (see `memory/AGENTS.md`).
    ///
    /// Caller must hold `PageMap::l1_mapping`.
    pub(super) fn assign(&self, segment: L2Segment, value: MapEntry) -> Result<(), PageMapError> {
        // SAFETY: caller holds `PageMap::l1_mapping`.
        let l2_mapping = unsafe { &mut *self.l2_mapping.get() };
        let occupied_pages = l2_mapping.try_add_pages(segment.pages())?;
        let table = self
            .l2_table_ref()
            .ok_or(PageMapError::MetadataAllocFailed)?;

        table.write_pages(segment, value)?;
        l2_mapping.set_occupied_pages(occupied_pages);

        Ok(())
    }

    /// Caller must hold `PageMap::l1_mapping`.
    pub(super) fn clear_segment(&self, segment: L2Segment) -> Result<(), PageMapError> {
        // SAFETY: caller holds `PageMap::l1_mapping`.
        let l2_mapping = unsafe { &mut *self.l2_mapping.get() };
        let occupied_pages = l2_mapping.try_sub_pages(segment.pages())?;
        let table = self.l2_table_ref().ok_or(PageMapError::UnexpectedEntry)?;

        table.write_pages(segment, MapEntry::empty())?;
        l2_mapping.set_occupied_pages(occupied_pages);

        Ok(())
    }
}

#[repr(C)]
pub(super) struct L2Table {
    pub(super) pages: [AtomicMapEntry; L2_ENTRIES],
}

impl L2Table {
    fn get(&self, index: L2Index) -> Option<MapEntry> {
        let page = self.pages.get(index.get())?.load();
        if page.is_empty() { None } else { Some(page) }
    }

    fn owns_segment(&self, segment: L2Segment, expected: MapEntry) -> Result<bool, PageMapError> {
        if expected.is_empty() {
            return Ok(self.segment_is_free(segment));
        }

        let pages = self
            .pages
            .get(segment.range())
            .ok_or(PageMapError::InvalidRange)?;

        Ok(pages.iter().all(|entry| entry.load() == expected))
    }

    fn segment_is_free(&self, segment: L2Segment) -> bool {
        let Some(pages) = self.pages.get(segment.range()) else {
            return false;
        };

        pages.iter().all(|entry| entry.load().is_empty())
    }

    fn write_pages(&self, segment: L2Segment, value: MapEntry) -> Result<(), PageMapError> {
        let entries = self
            .pages
            .get(segment.range())
            .ok_or(PageMapError::InvalidRange)?;

        for entry in entries {
            entry.store(value);
        }

        Ok(())
    }
}
