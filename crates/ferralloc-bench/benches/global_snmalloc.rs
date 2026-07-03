use criterion::{Criterion, criterion_group, criterion_main};

mod common;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

fn global_collections(c: &mut Criterion) {
    common::register_global_collections(c, "snmalloc");
}

criterion_group!(global_snmalloc, global_collections);
criterion_main!(global_snmalloc);
