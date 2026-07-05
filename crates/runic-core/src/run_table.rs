use core::{mem::MaybeUninit, ptr::NonNull, slice};

use crate::{
    layout::LayoutSpec,
    os_memory::OsMemory,
    run::{Run, RunId},
    size_class::SizeClass,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RunTableError {
    InvalidReservation,
    Occupied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RunReservation {
    id: RunId,
}

impl RunReservation {
    pub(crate) const fn id(self) -> RunId {
        self.id
    }
}

pub(crate) struct RunTable {
    slots: Option<RunSlots>,
    next: u32,
    capacity: u32,
}

impl RunTable {
    pub(crate) const fn new(capacity: u32) -> Self {
        Self {
            slots: None,
            next: 0,
            capacity,
        }
    }

    pub(crate) fn reserve(&mut self) -> Option<RunReservation> {
        let capacity = usize::try_from(self.capacity).ok()?;
        if capacity == 0 {
            return None;
        }
        let start = usize::try_from(self.next).ok()?;

        for offset in 0..capacity {
            let sum = start.checked_add(offset)?;
            let index = if sum >= capacity {
                sum.checked_sub(capacity)?
            } else {
                sum
            };

            let slot = self.slots_mut()?.get_mut(index)?;

            if slot.reserve() {
                let next = if index + 1 == capacity {
                    0
                } else {
                    index.checked_add(1)?
                };
                self.next = u32::try_from(next).ok()?;
                let id = RunId::from_index(u32::try_from(index).ok()?)?;

                return Some(RunReservation { id });
            }
        }

        None
    }

    pub(crate) fn release(&mut self, reservation: RunReservation) {
        let Some(slot) = self.slot_mut(reservation.id) else {
            return;
        };

        slot.release();
    }

    pub(crate) fn insert(
        &mut self,
        reservation: RunReservation,
        run: Run,
    ) -> Result<RunId, RunTableError> {
        if reservation.id != run.id() {
            self.release(reservation);
            return Err(RunTableError::InvalidReservation);
        }

        let Some(slot) = self.slot_mut(reservation.id) else {
            return Err(RunTableError::InvalidReservation);
        };

        if slot.insert(run) {
            Ok(reservation.id)
        } else {
            slot.release();
            Err(RunTableError::Occupied)
        }
    }

    pub(crate) fn get(&self, id: RunId) -> Option<&Run> {
        self.slot(id)?.get()
    }

    pub(crate) fn get_mut(&mut self, id: RunId) -> Option<&mut Run> {
        self.slot_mut(id)?.get_mut()
    }

    pub(crate) fn allocate(
        &mut self,
        class: SizeClass,
        spec: LayoutSpec,
    ) -> Option<(RunId, NonNull<u8>)> {
        for (index, slot) in self.slots.as_mut()?.slots_mut().iter_mut().enumerate() {
            let Some(run) = slot.get_mut() else {
                continue;
            };

            if run.class() != class.id() {
                continue;
            }

            let Some(ptr) = run.allocate(spec) else {
                continue;
            };
            let id = RunId::from_index(u32::try_from(index).ok()?)?;

            return Some((id, ptr));
        }

        None
    }

    pub(crate) fn remove(&mut self, id: RunId) -> Option<Run> {
        self.slot_mut(id)?.remove()
    }

    fn slots(&self) -> Option<&RunSlots> {
        self.slots.as_ref()
    }

    fn slots_mut(&mut self) -> Option<&mut RunSlots> {
        if self.slots.is_none() {
            self.slots = Some(RunSlots::new(usize::try_from(self.capacity).ok()?)?);
        }

        self.slots.as_mut()
    }

    fn slot(&self, id: RunId) -> Option<&RunSlot> {
        self.slots()?.get(Self::index(id)?)
    }

    fn slot_mut(&mut self, id: RunId) -> Option<&mut RunSlot> {
        self.slots.as_mut()?.get_mut(Self::index(id)?)
    }

    fn index(id: RunId) -> Option<usize> {
        usize::try_from(id.index()).ok()
    }
}

struct RunSlots {
    mapping: crate::os_memory::Mapping,
    slots: NonNull<RunSlot>,
}

// SAFETY: RunSlots owns mmap-backed slots. Moving ownership to another
// thread does not permit concurrent mutation of allocator metadata.
unsafe impl Send for RunSlots {}

impl RunSlots {
    fn new(len: usize) -> Option<Self> {
        let byte_len = len.checked_mul(core::mem::size_of::<RunSlot>())?;
        let mapping = OsMemory::map(byte_len)?;
        let slots = mapping.base().cast::<RunSlot>();

        Some(Self { mapping, slots })
    }

    fn get(&self, index: usize) -> Option<&RunSlot> {
        self.slots().get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut RunSlot> {
        self.slots_mut().get_mut(index)
    }

    fn slots(&self) -> &[RunSlot] {
        let len = self.mapping.range().len() / core::mem::size_of::<RunSlot>();

        // SAFETY: slots points to mmap storage sized for len RunSlot entries.
        unsafe { slice::from_raw_parts(self.slots.as_ptr(), len) }
    }

    fn slots_mut(&mut self) -> &mut [RunSlot] {
        let len = self.mapping.range().len() / core::mem::size_of::<RunSlot>();

        // SAFETY: RunSlots has unique access to the mmap storage here.
        unsafe { slice::from_raw_parts_mut(self.slots.as_ptr(), len) }
    }
}

impl Drop for RunSlots {
    fn drop(&mut self) {
        for slot in self.slots_mut() {
            slot.drop_run();
        }
    }
}

#[repr(C)]
struct RunSlot {
    run: MaybeUninit<Run>,
    state: SlotState,
}

impl RunSlot {
    fn reserve(&mut self) -> bool {
        if !self.state.is_empty() {
            return false;
        }

        self.state = SlotState::reserved();
        true
    }

    fn release(&mut self) {
        if self.state.is_reserved() {
            self.state = SlotState::empty();
        }
    }

    fn insert(&mut self, run: Run) -> bool {
        if !self.state.is_reserved() {
            return false;
        }

        self.run.write(run);
        self.state = SlotState::occupied();
        true
    }

    fn get(&self) -> Option<&Run> {
        if !self.state.is_occupied() {
            return None;
        }

        // SAFETY: occupied state is set only after run.write initializes the slot.
        Some(unsafe { self.run.assume_init_ref() })
    }

    fn get_mut(&mut self) -> Option<&mut Run> {
        if !self.state.is_occupied() {
            return None;
        }

        // SAFETY: occupied state is set only after run.write initializes the slot.
        Some(unsafe { self.run.assume_init_mut() })
    }

    fn remove(&mut self) -> Option<Run> {
        if !self.state.is_occupied() {
            return None;
        }

        self.state = SlotState::empty();

        // SAFETY: occupied state was true on entry, so the slot contains an initialized Run.
        Some(unsafe { self.run.assume_init_read() })
    }

    fn drop_run(&mut self) {
        let _ = self.remove();
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct SlotState {
    raw: u8,
}

impl SlotState {
    const EMPTY: u8 = 0;
    const RESERVED: u8 = 1;
    const OCCUPIED: u8 = 2;

    const fn empty() -> Self {
        Self { raw: Self::EMPTY }
    }

    const fn reserved() -> Self {
        Self {
            raw: Self::RESERVED,
        }
    }

    const fn occupied() -> Self {
        Self {
            raw: Self::OCCUPIED,
        }
    }

    const fn is_empty(self) -> bool {
        self.raw == Self::EMPTY
    }

    const fn is_reserved(self) -> bool {
        self.raw == Self::RESERVED
    }

    const fn is_occupied(self) -> bool {
        self.raw == Self::OCCUPIED
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        layout::LayoutSpec,
        os_memory::OsMemory,
        run::{RUN_SIZE, Run, RunId},
        size_class::SizeClasses,
    };

    use super::*;

    fn reusable_run(id: RunId) -> Run {
        let mapping = OsMemory::map(RUN_SIZE).unwrap();
        let spec = LayoutSpec::from_size_align(64, 8).unwrap();
        let class = SizeClasses::get(spec).unwrap();

        Run::new(id, mapping, class)
    }

    fn table_with_capacity(capacity: usize) -> RunTable {
        RunTable::new(u32::try_from(capacity).unwrap())
    }

    #[test]
    fn run_table_zero_capacity_reserves_none() {
        let mut table = RunTable::new(0);

        assert_eq!(table.reserve(), None);
    }

    #[test]
    fn run_table_reserves_ids_from_zero() {
        let mut table = table_with_capacity(4);

        assert_eq!(table.reserve().unwrap().id().index(), 0);
        assert_eq!(table.reserve().unwrap().id().index(), 1);
    }

    #[test]
    fn run_table_respects_injected_capacity() {
        let mut table = table_with_capacity(2);

        assert_eq!(table.reserve().unwrap().id().index(), 0);
        assert_eq!(table.reserve().unwrap().id().index(), 1);
        assert_eq!(table.reserve(), None);
    }

    #[test]
    fn run_table_release_makes_reserved_slot_available() {
        let mut table = table_with_capacity(4);
        let first = table.reserve().unwrap();
        let second = table.reserve().unwrap();

        table.release(first);

        assert_eq!(second.id().index(), 1);
        for expected in 2..4 {
            assert_eq!(table.reserve().unwrap().id().index(), expected);
        }
        assert_eq!(table.reserve().unwrap().id(), first.id());
    }

    #[test]
    fn run_table_insert_get_round_trip() {
        let mut table = table_with_capacity(4);
        let reservation = table.reserve().unwrap();
        let run = reusable_run(reservation.id());

        let id = table.insert(reservation, run).unwrap();
        assert_eq!(table.get(id).unwrap().id(), id);

        let run = table.remove(id).unwrap();
        assert_eq!(run.id(), id);
    }

    #[test]
    fn run_table_rejects_occupied_slot() {
        let mut table = table_with_capacity(4);
        let reservation = table.reserve().unwrap();
        let first = reusable_run(reservation.id());
        let second = reusable_run(reservation.id());

        let id = table.insert(reservation, first).unwrap();
        assert_eq!(
            table.insert(RunReservation { id }, second),
            Err(RunTableError::Occupied)
        );

        let _removed = table.remove(id);
    }

    #[test]
    fn run_table_rejects_unreserved_insert() {
        let mut table = table_with_capacity(4);
        let id = RunId::from_index(0).unwrap();
        let run = reusable_run(id);

        assert_eq!(
            table.insert(RunReservation { id }, run),
            Err(RunTableError::InvalidReservation)
        );
    }

    #[test]
    fn run_table_invalid_insert_releases_reservation() {
        let mut table = table_with_capacity(4);
        let reservation = table.reserve().unwrap();
        let released = reservation.id();
        let wrong_id = RunId::from_index(released.index() + 1).unwrap();
        let run = reusable_run(wrong_id);

        assert_eq!(
            table.insert(reservation, run),
            Err(RunTableError::InvalidReservation)
        );

        for expected in 1..4 {
            assert_eq!(table.reserve().unwrap().id().index(), expected);
        }
        assert_eq!(table.reserve().unwrap().id(), released);
    }

    #[test]
    fn run_table_get_mut_allows_run_mutation() {
        let mut table = table_with_capacity(4);
        let reservation = table.reserve().unwrap();
        let run = reusable_run(reservation.id());
        let spec = LayoutSpec::from_size_align(64, 8).unwrap();

        let id = table.insert(reservation, run).unwrap();
        let ptr = table.get_mut(id).unwrap().allocate(spec).unwrap();

        assert!(table.get_mut(id).unwrap().free(ptr).is_ok());

        let _removed = table.remove(id);
    }

    #[test]
    fn run_table_remove_clears_slot() {
        let mut table = table_with_capacity(4);
        let reservation = table.reserve().unwrap();
        let run = reusable_run(reservation.id());

        let id = table.insert(reservation, run).unwrap();
        assert!(table.remove(id).is_some());
        assert!(table.get(id).is_none());
        assert!(table.get_mut(id).is_none());
    }
}
