//! `Perf` measures the selected perf events using the perf interface of the Linux kernel.
//!
//! # Example
//!
//! ```rust
//! extern crate criterion_perf_events;
//! extern crate perfcnt;
//!
//! # fn fibonacci_slow(_: usize) {}
//! # fn fibonacci_fast(_: usize) {}
//!
//! use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
//! use criterion_perf_events::Perf;
//! use perfcnt::linux::HardwareEventType as Hardware;
//! use perfcnt::linux::PerfCounterBuilderLinux as Builder;
//!
//! fn bench(c: &mut Criterion<Perf>) {
//!     let mut group = c.benchmark_group("fibonacci");
//!
//!     for i in 0..20 {
//!         group.bench_function(BenchmarkId::new("slow", i), |b| b.iter(|| fibonacci_slow(i)));
//!         group.bench_function(BenchmarkId::new("fast", i), |b| b.iter(|| fibonacci_fast(i)));
//!     }
//!
//!     group.finish()
//! }
//!
//! criterion_group!(
//!     name = my_bench;
//!     config = Criterion::default().with_measurement(Perf::new(Builder::from_hardware_event(Hardware::CacheMisses)));
//!     targets = bench
//! );
//! criterion_main!(my_bench);
//! ```

extern crate perfcnt;

use criterion::{
    measurement::{Measurement, ValueFormatter},
    Throughput,
};
use std::cell::RefCell;

use perfcnt::linux::PerfCounter;
use perfcnt::linux::PerfCounterBuilderLinux;
use perfcnt::AbstractPerfCounter;

/// `perf` implements `criterion::measurement::Measurement` so it can be used in criterion to measure perf events.
/// Create a struct via `Perf::new()`.
pub struct Perf {
    counter: RefCell<PerfCounter>,
}

impl Perf {
    /// Creates a new criterion measurement plugin that measures perf events.
    ///
    /// # Argument
    ///
    /// * `builder` - A PerfCounterBuilderLinux from the crate perfcnt that is configured for the selected counter.
    ///
    /// # Remarks
    ///
    /// Should only fail if you select a counter that is not available on your system or you do not have the necessarry access rights.
    pub fn new(mut builder: PerfCounterBuilderLinux) -> Perf {
        Perf {
            counter: RefCell::new(
                builder
                    .for_pid(std::process::id() as i32)
                    .disable()
                    .finish()
                    .expect("Could not create counter"),
            ),
        }
    }
}

impl Measurement for Perf {
    type Intermediate = u64;
    type Value = u64;

    fn start(&self) -> Self::Intermediate {
        self.counter
            .borrow()
            .start()
            .expect("Could not read perf counter");
        0
    }

    fn end(&self, _i: Self::Intermediate) -> Self::Value {
        self.counter
            .borrow()
            .stop()
            .expect("Could not stop perf counter");
        let ret = self
            .counter
            .borrow_mut()
            .read()
            .expect("Could not read perf counter");
        self.counter
            .borrow_mut()
            .reset()
            .expect("Could not reset perf counter");
        ret
    }

    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        v1 + v2
    }

    fn zero(&self) -> Self::Value {
        0
    }

    fn to_f64(&self, value: &Self::Value) -> f64 {
        *value as f64
    }

    fn formatter(&self) -> &dyn ValueFormatter {
        &PerfFormatter
    }
}

struct PerfFormatter;

impl ValueFormatter for PerfFormatter {
    fn format_value(&self, value: f64) -> String {
        format!("{:.4} cycles", value)
    }

    fn format_throughput(&self, throughput: &Throughput, value: f64) -> String {
        match throughput {
            Throughput::Bytes(b) => format!("{:.4} events/byte", value / *b as f64),
            Throughput::BytesDecimal(b) => format!("{:.4} events/kilobyte", value / *b as f64),
            Throughput::Elements(b) => format!("{:.4} events/element", value / *b as f64),
        }
    }

    fn scale_values(&self, _typical_value: f64, _values: &mut [f64]) -> &'static str {
        "events"
    }

    fn scale_throughputs(
        &self,
        _typical_value: f64,
        throughput: &Throughput,
        values: &mut [f64],
    ) -> &'static str {
        match throughput {
            Throughput::Bytes(n) => {
                for val in values {
                    *val /= *n as f64;
                }
                "events/byte"
            }
            Throughput::BytesDecimal(n) => {
                for val in values {
                    *val /= *n as f64;
                }
                "events/kilobyte"
            }
            Throughput::Elements(n) => {
                for val in values {
                    *val /= *n as f64;
                }
                "events/element"
            }
        }
    }

    fn scale_for_machines(&self, _values: &mut [f64]) -> &'static str {
        "events"
    }
}
