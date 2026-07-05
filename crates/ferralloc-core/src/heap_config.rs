use crate::table_capacity::TableCapacity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HeapConfig {
    run_capacity: TableCapacity,
    extent_capacity: TableCapacity,
}

impl HeapConfig {
    const DEFAULT_TABLE_CAPACITY: TableCapacity = match TableCapacity::new(65_536) {
        Some(capacity) => capacity,
        None => panic!("default table capacity must be valid"),
    };

    pub(crate) const fn new(run_capacity: TableCapacity, extent_capacity: TableCapacity) -> Self {
        Self {
            run_capacity,
            extent_capacity,
        }
    }

    pub(crate) const fn default() -> Self {
        Self::new(Self::DEFAULT_TABLE_CAPACITY, Self::DEFAULT_TABLE_CAPACITY)
    }

    pub(crate) const fn run_capacity(self) -> TableCapacity {
        self.run_capacity
    }

    pub(crate) const fn extent_capacity(self) -> TableCapacity {
        self.extent_capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heap_config_stores_table_capacities() {
        let run_capacity = TableCapacity::new(3).unwrap();
        let extent_capacity = TableCapacity::new(5).unwrap();
        let config = HeapConfig::new(run_capacity, extent_capacity);

        assert_eq!(config.run_capacity(), run_capacity);
        assert_eq!(config.extent_capacity(), extent_capacity);
    }
}
