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
    c.bench_function(&format!("{name} select_nth"), |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| {
                let (p, _, _) = v.select_nth_unstable(prefix - 1);
                p.sort_unstable();
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function(&format!("{name} partial_sort_1_0_0"), |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| partial_sort_1_0_0::partial_sort(v, prefix, |a, b| a.lt(b)),
            BatchSize::SmallInput,
        );
    });

    c.bench_function(&format!("{name} vendored_std_1"), |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| {
                partial_sort_vendored_std::<_, _, _, 1>(v, ..prefix, |a, b| a.lt(b));
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function(&format!("{name} vendored_std_2"), |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| {
                partial_sort_vendored_std::<_, _, _, 2>(v, ..prefix, |a, b| a.lt(b));
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function(&format!("{name} vendored_std_4"), |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| {
                partial_sort_vendored_std::<_, _, _, 4>(v, ..prefix, |a, b| a.lt(b));
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function(&format!("{name} vendored_std_8"), |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| {
                partial_sort_vendored_std::<_, _, _, 8>(v, ..prefix, |a, b| a.lt(b));
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function(&format!("{name} vendored_std_16"), |b| {
        b.iter_batched_ref(
            setup.clone(),
            |v| {
                partial_sort_vendored_std::<_, _, _, 16>(v, ..prefix, |a, b| a.lt(b));
            },
            BatchSize::SmallInput,
        );
    });
}

fn random_u64(len: usize) -> Vec<u64> {
    let mut rng = rand::rng();
    (0..len).map(|_| rng.random::<u64>()).collect()
}

fn random_uniform_u64(len: usize, range: Range<u64>) -> Vec<u64>
where
{
    let mut rng = rand::rng();
    let dist = Uniform::try_from(range).expect("uniform");
    (0..len).map(|_| dist.sample(&mut rng)).collect()
}

fn random_x_percent_u64(len: usize, mid_percent: f64) -> Vec<u64> {
    let len_const = ((len as f64 / 100.0) * (100.0 - mid_percent)).round() as usize;
    let len_random = len - len_const;

    let mut v: Vec<u64> = iter::repeat(u64::MAX / 2)
        .take(len_const)
        .chain(random_u64(len_random))
        .collect();

    let mut rng = rand::rng();
    v.shuffle(&mut rng);
    v
}

fn random_zipf_u64(len: usize, exponent: f64) -> Vec<u64> {
    let mut rng = rand::rng();
    let dist = Zipf::new(len as f64, exponent).expect("zipf");
    (0..len).map(|_| dist.sample(&mut rng) as u64).collect()
}

fn random_sorted_u64(len: usize, sorted_percent: f64) -> Vec<u64> {
    let mut v = random_u64(len);
    let len_sorted = ((len as f64) * (sorted_percent / 100.0)).round() as usize;
    v[..len_sorted].sort_unstable();
    v
}

fn benchmark_u64(c: &mut Criterion) {
    let scenarios_u64: &[(&str, fn(usize) -> Vec<u64>)] = &[
        ("random_uniform_u64", |len| {
            random_uniform_u64(len, 0..u64::MAX)
        }),
        ("random_x_percent_u64", |len| {
            random_x_percent_u64(len, 50.0)
        }),
    ];

    let lengths = [2, 4, 8, 100, 200];

    for len in lengths {
        for prefix in lengths {
            if prefix <= len {
                benchmark_algorithms(
                    c,
                    &format!("random_u64 len_{len} prefix_{prefix}"),
                    || random_u64(len),
                    prefix,
                );
            }
        }
    }
}

criterion_group!(benches, benchmark_u64);
criterion_main!(benches);
