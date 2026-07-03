use criterion::{Criterion, criterion_group, criterion_main};

mod common;

fn global_collections(c: &mut Criterion) {
    common::register_global_collections(c, "system");
}

criterion_group!(global_system, global_collections);
criterion_main!(global_system);
