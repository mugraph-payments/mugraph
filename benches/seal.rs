use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mugraph::protocol::*;
use prop::{strategy::ValueTree, test_runner::TestRunner};
use proptest::{prelude::*, strategy::Strategy};

fn bench_note(c: &mut Criterion) {
    let mut runner = TestRunner::new(ProptestConfig::default());

    let mut group = c.benchmark_group("Note");

    group.bench_function("circuit", |b| b.iter(|| black_box(Note::circuit())));

    let note: Note = any::<Note>().new_tree(&mut runner).unwrap().current();

    group.bench_with_input("prove", &note, |b, note| b.iter(|| note.prove().unwrap()));
    group.bench_with_input("seal", &note, |b, note| b.iter(|| note.seal().unwrap()));
    group.bench_with_input(
        "verify",
        &(note.hash(), note.seal().unwrap()),
        |b, (hash, seal)| b.iter(|| Note::verify(*hash, seal.clone())),
    );
}

fn bench_append<const I: usize, const O: usize>(c: &mut Criterion) {
    let mut runner = TestRunner::new(ProptestConfig::default());
    let append = any::<Append<I, O>>()
        .new_tree(&mut runner)
        .unwrap()
        .current();

    let mut group = c.benchmark_group(format!("Append<{I}, {O}>"));

    group.bench_function("circuit", |b| {
        b.iter(|| black_box(Append::<I, O>::circuit()))
    });

    group.bench_with_input("prove", &append, |b, append| {
        b.iter(|| append.prove().unwrap())
    });

    group.bench_with_input("seal", &append, |b, append| {
        b.iter(|| append.seal().unwrap())
    });

    group.bench_with_input(
        "verify",
        &(append.payload(), append.seal().unwrap()),
        |b, (payload, seal)| b.iter(move || Append::<I, O>::verify(payload.clone(), seal.clone())),
    );
}

fn benchmark(c: &mut Criterion) {
    bench_note(c);
    bench_append::<8, 8>(c);
    bench_append::<16, 16>(c);
    bench_append::<32, 32>(c);
    bench_append::<64, 64>(c);
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
