use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ferralloc_bench::{allocator_target::TARGETS, threaded};

const THREAD_COUNTS: &[usize] = &[2, 4];
const OPS_PER_THREAD: usize = 512;

fn configure_group(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    group
        .sample_size(10)
        .warm_up_time(Duration::from_millis(250))
        .measurement_time(Duration::from_secs(1));
}

fn thread_local_churn(c: &mut Criterion) {
    let mut group = c.benchmark_group("threaded/thread_local_churn");
    configure_group(&mut group);

    for &target in TARGETS {
        for &threads in THREAD_COUNTS {
            group.throughput(Throughput::Elements((threads * OPS_PER_THREAD) as u64));
            group.bench_with_input(
                BenchmarkId::new(target.name(), threads),
                &(target, threads),
                |bench, &(target, threads)| {
                    bench.iter(|| threaded::thread_local_churn(target, threads, OPS_PER_THREAD));
                },
            );
        }
    }

    group.finish();
}

fn cross_thread_free_ring(c: &mut Criterion) {
    let mut group = c.benchmark_group("threaded/cross_thread_free_ring");
    configure_group(&mut group);

    for &target in TARGETS {
        for &threads in THREAD_COUNTS {
            group.throughput(Throughput::Elements((threads * OPS_PER_THREAD) as u64));
            group.bench_with_input(
                BenchmarkId::new(target.name(), threads),
                &(target, threads),
                |bench, &(target, threads)| {
                    bench
                        .iter(|| threaded::cross_thread_free_ring(target, threads, OPS_PER_THREAD));
                },
            );
        }
    }

    group.finish();
}

fn mixed_thread_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("threaded/mixed_thread_random");
    configure_group(&mut group);

    for &target in TARGETS {
        for &threads in THREAD_COUNTS {
            group.throughput(Throughput::Elements((threads * OPS_PER_THREAD) as u64));
            group.bench_with_input(
                BenchmarkId::new(target.name(), threads),
                &(target, threads),
                |bench, &(target, threads)| {
                    bench.iter(|| threaded::mixed_thread_random(target, threads, OPS_PER_THREAD));
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    threaded_benches,
    thread_local_churn,
    cross_thread_free_ring,
    mixed_thread_random
);
criterion_main!(threaded_benches);
