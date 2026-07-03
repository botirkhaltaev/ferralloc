use criterion::{Criterion, criterion_group, criterion_main};

mod common;

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn global_collections(c: &mut Criterion) {
    common::register_global_collections(c, "jemalloc");
}

criterion_group!(global_jemalloc, global_collections);
criterion_main!(global_jemalloc);
