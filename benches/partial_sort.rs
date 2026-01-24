use rand_distr::Zipf;
use std::iter;
use std::ops::Range;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use partial_sort_vendored_std::partial_sort as partial_sort_vendored_std;
use rand::{distr::Uniform, prelude::*};

fn benchmark_algorithms<T, F>(c: &mut Criterion, name: &str, setup: F, prefix: usize)
where
    T: Ord,
    F: FnMut() -> Vec<T> + Clone,
{
    let mut group = c.benchmark_group(name);

    group.bench_function("select_nth", |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| {
                let (p, _, _) = v.select_nth_unstable(prefix - 1);
                p.sort_unstable();
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("partial_sort_1_0_0", |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| partial_sort_1_0_0::partial_sort(v, prefix, |a, b| a.lt(b)),
            BatchSize::SmallInput,
        );
    });

    group.bench_function("vendored_std_1", |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| {
                partial_sort_vendored_std::<_, _, _, 1>(v, ..prefix, |a, b| a.lt(b));
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("vendored_std_2", |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| {
                partial_sort_vendored_std::<_, _, _, 2>(v, ..prefix, |a, b| a.lt(b));
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("vendored_std_4", |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| {
                partial_sort_vendored_std::<_, _, _, 4>(v, ..prefix, |a, b| a.lt(b));
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("vendored_std_8", |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| {
                partial_sort_vendored_std::<_, _, _, 8>(v, ..prefix, |a, b| a.lt(b));
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("vendored_std_16", |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| {
                partial_sort_vendored_std::<_, _, _, 16>(v, ..prefix, |a, b| a.lt(b));
            },
            BatchSize::SmallInput,
        );
    });
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

fn benchmark_u64(c: &mut Criterion) {
    let scenarios: &[(&str, fn(usize) -> Vec<u64>)] = &[
        ("random", u64_random),
        ("random_z1", |len| u64_random_zipf(len, 1.0)),
        ("random_d20", |len| u64_random_uniform(len, 0..20)),
        ("random_p5", |len| u64_random_x_percent(len, 5.0)),
        ("random_p95", |len| u64_random_x_percent(len, 95.0)),
        ("ascending", u64_ascending),
        ("descending", u64_descending),
    ];

    let lengths = [2, 4, 8, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000];

    for (scenario_name, scenario_setup) in scenarios {
        for len in lengths {
            for prefix in lengths {
                if prefix <= len {
                    benchmark_algorithms(
                        c,
                        &format!("u64_{scenario_name}-len_{len}-prefix_{prefix}"),
                        || scenario_setup(len),
                        prefix,
                    );
                }
            }
        }
    }
}

criterion_group!(benches, benchmark_u64);
criterion_main!(benches);
