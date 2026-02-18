use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use criterion_perf_events::Perf;
use perfcnt::linux::HardwareEventType as Hardware;
use perfcnt::linux::PerfCounterBuilderLinux as Builder;
use std::hint::black_box;

fn fibonacci_slow(n: usize) -> usize {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci_slow(n - 1) + fibonacci_slow(n - 2),
    }
}

fn fibonacci_fast(n: usize) -> usize {
    let mut a = 0;
    let mut b = 1;

    match n {
        0 => b,
        _ => {
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}

fn bench(c: &mut Criterion<Perf>) {
    let mut group = c.benchmark_group("fibonacci");

    let fibo_arg = 30;
    group.bench_function(BenchmarkId::new("slow", fibo_arg), |b| {
        b.iter(|| fibonacci_slow(black_box(fibo_arg)))
    });
    group.bench_function(BenchmarkId::new("fast", fibo_arg), |b| {
        b.iter(|| fibonacci_fast(black_box(fibo_arg)))
    });

    group.finish()
}

criterion_group!(
    name = instructions_bench;
    config = Criterion::default().with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)));
    targets = bench
);
criterion_main!(instructions_bench);
