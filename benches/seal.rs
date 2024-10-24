use std::any::type_name;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mugraph::protocol::*;
use prop::{strategy::ValueTree, test_runner::TestRunner};
use proptest::{prelude::*, strategy::Strategy};

fn benchmark_sealable<T: Sealable + Arbitrary>(c: &mut Criterion) {
    let type_name = type_name::<T>();
    let group_name = type_name.split("::").last().unwrap_or(type_name);
    let mut runner = TestRunner::new(ProptestConfig::default());

    let mut group = c.benchmark_group(group_name);

    group.bench_function("circuit", |b| b.iter(|| black_box(T::circuit())));

    group.bench_function("prove", |b| {
        let note = any::<T>().new_tree(&mut runner).unwrap().current();
        b.iter(|| note.prove().unwrap())
    });

    group.bench_function("seal", |b| {
        let note = any::<T>().new_tree(&mut runner).unwrap().current();
        b.iter(|| note.seal().unwrap())
    });

    group.bench_function("verify", |b| {
        let note = any::<T>().new_tree(&mut runner).unwrap().current();
        let seal = note.seal().unwrap();
        b.iter(|| T::verify(note.hash(), seal.clone()))
    });

    group.bench_function("seal+verify", |b| {
        let note = any::<T>().new_tree(&mut runner).unwrap().current();
        b.iter(|| T::verify(note.hash(), note.seal().unwrap()))
    });
}

fn benchmark(c: &mut Criterion) {
    benchmark_sealable::<Note>(c);
    benchmark_sealable::<Append<2, 2>>(c);
    benchmark_sealable::<Append<4, 4>>(c);
    benchmark_sealable::<Append<8, 8>>(c);
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
