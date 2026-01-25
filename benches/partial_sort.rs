use criterion::BenchmarkId;
use rand_distr::Zipf;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::fmt;
use std::iter;
use std::ops::Range;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use rand::{distr::Uniform, prelude::*};

#[derive(Debug, Copy, Clone)]
struct Input {
    len: usize,
    prefix: usize,
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "len_{}-prefix_{}", self.len, self.prefix)
    }
}

fn benchmark_algorithms<T, F>(c: &mut Criterion, name: &str, mut setup: F)
where
    T: Ord,
    F: FnMut(usize) -> Vec<T> + Clone,
{
    let mut group = c.benchmark_group(name);

    let lengths = [1, 2, 3, 4, 5, 8, 10, 12, 15, 18, 20, 25, 30, 40, 50, 100, 200, 300, 400, 500, 1000, 2000, 3000, 4000, 5000, 10000];

    for len in lengths {
        for prefix in lengths {
            if prefix <= len {
                let input = Input { len, prefix };

                group.bench_with_input(BenchmarkId::new("select_nth", input), &input, |b, _| {
                    b.iter_batched_ref(
                        || setup(len),
                        |v| {
                            let (p, _, _) = v.select_nth_unstable(prefix - 1);
                            p.sort_unstable();
                        },
                        BatchSize::SmallInput,
                    );
                });

                group.bench_with_input(
                    BenchmarkId::new("partial_sort_1_0_0", input),
                    &input,
                    |b, _| {
                        b.iter_batched_ref(
                            || setup(len),
                            |v| partial_sort_1_0_0::partial_sort(v, prefix, |a, b| a.lt(b)),
                            BatchSize::SmallInput,
                        );
                    },
                );

                group.bench_with_input(
                    BenchmarkId::new("vendored_std", input),
                    &input,
                    |b, _| {
                        b.iter_batched_ref(
                            || setup(len),
                            |v| {
                                partial_sort_vendored_std::partial_sort(v, ..prefix, |a, b| {
                                    a.lt(b)
                                });
                            },
                            BatchSize::SmallInput,
                        );
                    },
                );
            }
        }
    }

    group.finish();
}

fn u64_random(len: usize) -> Vec<u64> {
    let mut rng = rand::rng();
    (0..len).map(|_| rng.random::<u64>()).collect()
}

fn u64_random_uniform(len: usize, range: Range<u64>) -> Vec<u64>
where
{
    let mut rng = rand::rng();
    let dist = Uniform::try_from(range).expect("uniform");
    (0..len).map(|_| dist.sample(&mut rng)).collect()
}

fn u64_random_x_percent(len: usize, mid_percent: f64) -> Vec<u64> {
    let len_const = ((len as f64 / 100.0) * (100.0 - mid_percent)).round() as usize;
    let len_random = len - len_const;

    let mut v: Vec<u64> = iter::repeat(u64::MAX / 2)
        .take(len_const)
        .chain(u64_random(len_random))
        .collect();

    let mut rng = rand::rng();
    v.shuffle(&mut rng);
    v
}

fn u64_random_zipf(len: usize, exponent: f64) -> Vec<u64> {
    let mut rng = rand::rng();
    let dist = Zipf::new(len as f64, exponent).expect("zipf");
    (0..len).map(|_| dist.sample(&mut rng) as u64).collect()
}

fn u64_ascending(len: usize) -> Vec<u64> {
    (0..len as u64).collect()
}

fn u64_descending(len: usize) -> Vec<u64> {
    (0..len as u64).rev().collect()
}

#[repr(C)]
struct KibibyteBlock {
    data: [u64; 128],
}

impl KibibyteBlock {
    fn new(mut n: u64) -> Self {
        let mut data = [0; 128];
        for cell in &mut data {
            *cell = n;
            n = n.wrapping_add(1);
        }
        Self { data }
    }
}

impl Ord for KibibyteBlock {
    fn cmp(&self, other: &Self) -> Ordering {
        self.data[0].cmp(&other.data[0])
    }
}

impl PartialOrd for KibibyteBlock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for KibibyteBlock {
    fn eq(&self, other: &Self) -> bool {
        self.data[0] == other.data[0]
    }
}

impl Eq for KibibyteBlock {}

fn benchmark_all(c: &mut Criterion) {
    let scenarios: &[(&str, fn(usize) -> Vec<u64>)] = &[
        ("random", u64_random),
        ("random_z1", |len| u64_random_zipf(len, 1.0)),
        ("random_d20", |len| u64_random_uniform(len, 0..20)),
        ("random_p5", |len| u64_random_x_percent(len, 5.0)),
        ("random_p95", |len| u64_random_x_percent(len, 95.0)),
        ("ascending", u64_ascending),
        ("descending", u64_descending),
    ];

    for (scenario_name, scenario_setup) in scenarios {
        benchmark_algorithms(c, &format!("u64_{scenario_name}"), scenario_setup);

        benchmark_algorithms(c, &format!("string_{scenario_name}"), |len| {
            scenario_setup(len)
                .into_iter()
                .map(|x| format!("{:10}", x))
                .collect()
        });

        benchmark_algorithms(c, &format!("kibibyte_{scenario_name}"), |len| {
            scenario_setup(len)
                .into_iter()
                .map(KibibyteBlock::new)
                .collect()
        });
    }
}

criterion_group!(benches, benchmark_all);
criterion_main!(benches);
