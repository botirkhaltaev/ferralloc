#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TableCapacity(usize);

impl TableCapacity {
    const MAX: usize = 4_294_967_295;

    pub(crate) const fn new(value: usize) -> Option<Self> {
        if value == 0 || value > Self::MAX {
            None
        } else {
            Some(Self(value))
        }
    }

    pub(crate) const fn get(self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_capacity_accepts_positive_value() {
        assert_eq!(TableCapacity::new(1).unwrap().get(), 1);
    }

    #[test]
    fn table_capacity_rejects_zero() {
        assert_eq!(TableCapacity::new(0), None);
    }
}
