use criterion::{Criterion, criterion_group, criterion_main};

mod common;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn global_collections(c: &mut Criterion) {
    common::register_global_collections(c, "mimalloc");
}

criterion_group!(global_mimalloc, global_collections);
criterion_main!(global_mimalloc);
