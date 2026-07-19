use crate::slot_store::{SlotStore, SlotStoreError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArenaError {
    InvalidReservation,
    Occupied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArenaReservation<Id> {
    pub(crate) id: Id,
}

pub(crate) trait ArenaId: Copy + Eq {
    fn index(self) -> u32;
    fn from_index(index: u32) -> Option<Self>;
}

pub(crate) trait ArenaValue<Id: ArenaId> {
    fn id(&self) -> Id;
}

pub(crate) struct Arena<T, Id>
where
    Id: ArenaId,
{
    slots: SlotStore<T>,
    _id: core::marker::PhantomData<Id>,
}

impl<T, Id> Arena<T, Id>
where
    Id: ArenaId,
    T: ArenaValue<Id>,
{
    pub(crate) const fn new(capacity: u32) -> Self {
        Self {
            slots: SlotStore::new(capacity),
            _id: core::marker::PhantomData,
        }
    }

    pub(crate) fn reserve(&mut self) -> Option<ArenaReservation<Id>> {
        let index = self.slots.reserve()?;
        let Some(id) = Self::id(index) else {
            let _ = self.slots.release(index);
            return None;
        };

        Some(ArenaReservation { id })
    }

    pub(crate) fn release(&mut self, reservation: ArenaReservation<Id>) {
        let Some(index) = Self::index(reservation.id) else {
            return;
        };

        let _ = self.slots.release(index);
    }

    pub(crate) fn insert(
        &mut self,
        reservation: ArenaReservation<Id>,
        value: T,
    ) -> Result<Id, ArenaError> {
        if reservation.id != value.id() {
            self.release(reservation);
            return Err(ArenaError::InvalidReservation);
        }

        let Some(index) = Self::index(reservation.id) else {
            return Err(ArenaError::InvalidReservation);
        };

        self.slots.insert(index, value).map_err(ArenaError::from)?;

        Ok(reservation.id)
    }

    pub(crate) fn get_mut(&mut self, id: Id) -> Option<&mut T>
    where
        Id: ArenaId,
    {
        self.slots.get_mut(Self::index(id)?)
    }

    pub(crate) fn remove(&mut self, id: Id) -> Option<T>
    where
        Id: ArenaId,
    {
        self.slots.remove(Self::index(id)?)
    }

    fn index(id: Id) -> Option<usize>
    where
        Id: ArenaId,
    {
        usize::try_from(id.index()).ok()
    }

    fn id(index: usize) -> Option<Id>
    where
        Id: ArenaId,
    {
        Id::from_index(u32::try_from(index).ok()?)
    }
}

impl From<SlotStoreError> for ArenaError {
    fn from(error: SlotStoreError) -> Self {
        match error {
            SlotStoreError::InvalidIndex | SlotStoreError::NotReserved => Self::InvalidReservation,
            SlotStoreError::Occupied => Self::Occupied,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct TestId(u32);

    impl ArenaId for TestId {
        fn index(self) -> u32 {
            self.0
        }

        fn from_index(index: u32) -> Option<Self> {
            Some(TestId(index))
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct TestValue(TestId);

    impl ArenaValue<TestId> for TestValue {
        fn id(&self) -> TestId {
            self.0
        }
    }

    fn arena_with_capacity(capacity: usize) -> Arena<TestValue, TestId> {
        Arena::new(u32::try_from(capacity).unwrap())
    }

    #[test]
    fn arena_zero_capacity_reserves_none() {
        let mut arena: Arena<TestValue, TestId> = Arena::new(0);
        assert_eq!(arena.reserve(), None);
    }

    #[test]
    fn arena_reserves_ids_from_zero() {
        let mut arena = arena_with_capacity(4);
        assert_eq!(arena.reserve().unwrap().id.0, 0);
        assert_eq!(arena.reserve().unwrap().id.0, 1);
    }

    #[test]
    fn arena_respects_injected_capacity() {
        let mut arena = arena_with_capacity(2);
        assert_eq!(arena.reserve().unwrap().id.0, 0);
        assert_eq!(arena.reserve().unwrap().id.0, 1);
        assert_eq!(arena.reserve(), None);
    }

    #[test]
    fn arena_release_makes_reserved_slot_available() {
        let mut arena = arena_with_capacity(4);
        let first = arena.reserve().unwrap();
        let second = arena.reserve().unwrap();
        arena.release(first);
        assert_eq!(second.id.0, 1);
        for expected in 2..4 {
            assert_eq!(arena.reserve().unwrap().id.0, expected);
        }
        assert_eq!(arena.reserve().unwrap().id.0, 0);
    }

    #[test]
    fn arena_insert_get_round_trip() {
        let mut arena = arena_with_capacity(4);
        let reservation = arena.reserve().unwrap();
        let value = TestValue(reservation.id);
        let id = arena.insert(reservation, value).unwrap();
        assert_eq!(arena.get_mut(id).unwrap().0, id);
        assert_eq!(arena.remove(id).unwrap().0, id);
    }

    #[test]
    fn arena_rejects_occupied_slot() {
        let mut arena = arena_with_capacity(4);
        let reservation = arena.reserve().unwrap();
        let first = TestValue(reservation.id);
        let second = TestValue(reservation.id);
        let id = arena.insert(reservation, first).unwrap();
        assert_eq!(
            arena.insert(ArenaReservation { id }, second),
            Err(ArenaError::Occupied)
        );
        let _removed = arena.remove(id);
    }

    #[test]
    fn arena_rejects_unreserved_insert() {
        let mut arena = arena_with_capacity(4);
        let id = TestId(0);
        let value = TestValue(TestId(1));
        assert_eq!(
            arena.insert(ArenaReservation { id }, value),
            Err(ArenaError::InvalidReservation)
        );
    }

    #[test]
    fn arena_invalid_insert_releases_reservation() {
        let mut arena = arena_with_capacity(4);
        let reservation = arena.reserve().unwrap();
        let reserved_id = reservation.id;
        // Create a value with a DIFFERENT id than the reservation
        let wrong_value = TestValue(TestId(reserved_id.0 + 1));
        // This insert should fail because reservation.id != value.id()
        assert_eq!(
            arena.insert(reservation, wrong_value),
            Err(ArenaError::InvalidReservation)
        );
        // The reservation should have been released, so we can reserve the same slot again
        for expected in 1..4 {
            assert_eq!(arena.reserve().unwrap().id.0, expected);
        }
        assert_eq!(arena.reserve().unwrap().id.0, 0);
    }
}
