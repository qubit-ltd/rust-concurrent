/*!
 * Benchmark for [`qubit_concurrent::task::service::ThreadPool`].
 */

use std::convert::Infallible;

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use qubit_concurrent::task::service::{ExecutorService, ThreadPool};

/// Runs one batch of no-op tasks and waits until the pool terminates.
fn run_noop_batch(pool_size: usize, task_count: usize) {
    let pool = ThreadPool::new(pool_size).expect("thread pool should be created");
    let mut handles = Vec::with_capacity(task_count);
    for _ in 0..task_count {
        let handle = pool
            .submit_callable(|| Ok::<usize, Infallible>(1))
            .expect("task should be accepted");
        handles.push(handle);
    }
    let mut sum = 0usize;
    for handle in handles {
        sum += handle.get().expect("task should complete");
    }
    black_box(sum);
    pool.shutdown();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be created")
        .block_on(pool.await_termination());
}

/// Runs one batch of light CPU tasks and waits until the pool terminates.
fn run_cpu_light_batch(pool_size: usize, task_count: usize) {
    run_cpu_work_batch(pool_size, task_count, 128);
}

/// Runs one batch of CPU tasks with configurable per-task work and waits until
/// the pool terminates.
fn run_cpu_work_batch(pool_size: usize, task_count: usize, inner_iters: usize) {
    let pool = ThreadPool::new(pool_size).expect("thread pool should be created");
    let mut handles = Vec::with_capacity(task_count);
    for _ in 0..task_count {
        let iterations = inner_iters;
        let handle = pool
            .submit_callable(move || {
                let mut acc = 0usize;
                for i in 0..iterations {
                    acc = acc.wrapping_add(black_box(i));
                }
                Ok::<usize, Infallible>(acc)
            })
            .expect("task should be accepted");
        handles.push(handle);
    }
    let mut sum = 0usize;
    for handle in handles {
        sum = sum.wrapping_add(handle.get().expect("task should complete"));
    }
    black_box(sum);
    pool.shutdown();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be created")
        .block_on(pool.await_termination());
}

/// Benchmarks throughput under different worker counts and task types.
fn bench_thread_pool_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("thread_pool_throughput");
    let workers = [1usize, 2, 4, 8];
    let task_count = 2_000usize;
    group.throughput(Throughput::Elements(task_count as u64));
    for worker_count in workers {
        group.bench_with_input(
            BenchmarkId::new("noop_tasks", worker_count),
            &worker_count,
            |b, &wc| b.iter(|| run_noop_batch(wc, task_count)),
        );
        group.bench_with_input(
            BenchmarkId::new("cpu_light_tasks", worker_count),
            &worker_count,
            |b, &wc| b.iter(|| run_cpu_light_batch(wc, task_count)),
        );
    }
    group.finish();
}

/// Benchmarks scheduling overhead vs task granularity under fixed total work.
fn bench_thread_pool_granularity(c: &mut Criterion) {
    let mut group = c.benchmark_group("thread_pool_granularity");
    let workers = [1usize, 4, 8];
    let granularities = [32usize, 256, 2_048];
    let total_iters = 2_048_000usize;
    for worker_count in workers {
        for inner_iters in granularities {
            let task_count = total_iters / inner_iters;
            let id = format!("workers={worker_count}/iters={inner_iters}");
            group.throughput(Throughput::Elements(task_count as u64));
            group.bench_with_input(BenchmarkId::from_parameter(id), &worker_count, |b, &wc| {
                b.iter(|| run_cpu_work_batch(wc, task_count, inner_iters))
            });
        }
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = bench_thread_pool_throughput, bench_thread_pool_granularity
);
criterion_main!(benches);
