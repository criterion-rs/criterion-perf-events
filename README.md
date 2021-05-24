# Criterion-perf-events

This is a measurement plugin for [Criterion.rs](https://github.com/bheisler/criterion.rs) to measure events of the Linux perf interface.

## Supported Events

Criterion-perf-events uses the [`perfcnt`](https://gz.github.io/rust-perfcnt/perfcnt/index.html) crate and supports events provided by this crate. If you are interested in more details, please take a look at the events listed here:

* [Hardware events](https://gz.github.io/rust-perfcnt/perfcnt/linux/enum.HardwareEventType.html)
* [Software events](https://gz.github.io/rust-perfcnt/perfcnt/linux/enum.SoftwareEventType.html)
* [Perf events](https://gz.github.io/rust-perfcnt/perfcnt/linux/enum.Event.html)
* [Raw Intel events](https://gz.github.io/rust-perfcnt/x86/perfcnt/intel/description/struct.IntelPerformanceCounterDescription.html)

## Troubleshooting

If you get a "Permission denied" error, update `perf_event_paranoid`:
```
sudo sh -c 'echo 1 >/proc/sys/kernel/perf_event_paranoid'
```
For further details please take a look at the following [link](https://superuser.com/questions/980632/run-perf-without-root-rights).

## Example

The following code shows how to count retired instructions.

```rust
use criterion::{criterion_group, criterion_main, BenchmarkId, black_box, Criterion};
use criterion_perf_events::Perf;
use perfcnt::linux::HardwareEventType as Hardware;
use perfcnt::linux::PerfCounterBuilderLinux as Builder;

fn fibonacci_slow(n: usize) -> usize {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci_slow(n - 1) + fibonacci_slow(n - 2),
    }
}

fn bench(c: &mut Criterion<Perf>) {
    let mut group = c.benchmark_group("fibonacci");

    let fibo_arg = 30;
    group.bench_function(BenchmarkId::new("slow", fibo_arg), |b| {
        b.iter(|| fibonacci_slow(black_box(fibo_arg)))
    });

    group.finish()
}

criterion_group!(
    name = instructions_bench;
    config = Criterion::default().with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)));
    targets = bench
);
criterion_main!(instructions_bench);
```

run with:
```
cargo criterion
```
Open `target/criterion/reports/index.html` to view detailed results with plots.
For all event types (`Hardware::Instructions`, `Hardware::CacheMisses`...) criterion will always report cycles as the unit.
Note that your event type is what is being shown, not CPU cycles.
