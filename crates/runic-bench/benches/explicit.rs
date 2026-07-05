use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use runic_bench::{allocator_target::TARGETS, workload};

const SMALL_OPS: usize = 512;
const RANDOM_OPS: usize = 2_000;
const RANDOM_LIVE: usize = 256;
const LARGE_OPS: usize = 64;
const REALLOC_ROUNDS: usize = 16;

fn configure_group(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    group
        .sample_size(10)
        .warm_up_time(Duration::from_millis(250))
        .measurement_time(Duration::from_secs(1));
}

fn single_size_churn(c: &mut Criterion) {
    let mut group = c.benchmark_group("explicit/single_size_churn");
    configure_group(&mut group);
    group.throughput(Throughput::Elements(SMALL_OPS as u64));

    for &target in TARGETS {
        for &size in workload::SINGLE_SIZE_CHURN {
            group.bench_with_input(
                BenchmarkId::new(target.name(), size),
                &(target, size),
                |bench, &(target, size)| {
                    bench.iter(|| workload::single_size_churn(target, size, SMALL_OPS));
                },
            );
        }
    }

    group.finish();
}

fn size_boundary_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("explicit/size_boundary_sweep");
    configure_group(&mut group);
    group.throughput(Throughput::Elements(SMALL_OPS as u64));

    for &target in TARGETS {
        group.bench_function(target.name(), |bench| {
            bench.iter(|| workload::size_boundary_sweep(target, SMALL_OPS));
        });
    }

    group.finish();
}

fn small_biased_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("explicit/small_biased_random");
    configure_group(&mut group);
    group.throughput(Throughput::Elements(RANDOM_OPS as u64));

    for &target in TARGETS {
        group.bench_function(target.name(), |bench| {
            bench.iter(|| {
                workload::small_biased_random(
                    target,
                    0xf3ee_a110_c001_cafe,
                    RANDOM_OPS,
                    RANDOM_LIVE,
                )
            });
        });
    }

    group.finish();
}

fn alignment_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("explicit/alignment_stress");
    configure_group(&mut group);
    group.throughput(Throughput::Elements(SMALL_OPS as u64));

    for &target in TARGETS {
        for &(size, align) in workload::ALIGNMENT_CASES {
            group.bench_with_input(
                BenchmarkId::new(target.name(), format!("size_{size}_align_{align}")),
                &(target, size, align),
                |bench, &(target, size, align)| {
                    bench.iter(|| workload::alignment_stress(target, size, align, SMALL_OPS));
                },
            );
        }
    }

    group.finish();
}

fn realloc_growth(c: &mut Criterion) {
    let mut group = c.benchmark_group("explicit/realloc_growth");
    configure_group(&mut group);
    group.throughput(Throughput::Elements(REALLOC_ROUNDS as u64));

    for &target in TARGETS {
        group.bench_function(target.name(), |bench| {
            bench.iter(|| workload::realloc_growth(target, REALLOC_ROUNDS));
        });
    }

    group.finish();
}

fn large_alloc_churn(c: &mut Criterion) {
    let mut group = c.benchmark_group("explicit/large_alloc_churn");
    configure_group(&mut group);
    group.throughput(Throughput::Elements(LARGE_OPS as u64));

    for &target in TARGETS {
        for &size in workload::LARGE_SIZES {
            group.bench_with_input(
                BenchmarkId::new(target.name(), size),
                &(target, size),
                |bench, &(target, size)| {
                    bench.iter(|| workload::large_alloc_churn(target, size, LARGE_OPS));
                },
            );
        }
    }

    group.finish();
}

fn alloc_zeroed(c: &mut Criterion) {
    let mut group = c.benchmark_group("explicit/alloc_zeroed");
    configure_group(&mut group);
    group.throughput(Throughput::Elements(SMALL_OPS as u64));

    for &target in TARGETS {
        for &size in &[64, 4096, 64 * 1024] {
            group.bench_with_input(
                BenchmarkId::new(target.name(), size),
                &(target, size),
                |bench, &(target, size)| {
                    bench.iter(|| workload::alloc_zeroed(target, size, SMALL_OPS));
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    explicit,
    single_size_churn,
    size_boundary_sweep,
    small_biased_random,
    alignment_stress,
    realloc_growth,
    large_alloc_churn,
    alloc_zeroed
);
criterion_main!(explicit);
