use criterion::{Criterion, criterion_group, criterion_main};

mod common;

#[global_allocator]
static ALLOC: ferralloc::Ferralloc = ferralloc::Ferralloc;

fn global_collections(c: &mut Criterion) {
    common::register_global_collections(c, "ferralloc");
}

criterion_group!(global_ferralloc, global_collections);
criterion_main!(global_ferralloc);
