use criterion::{criterion_group, criterion_main, Criterion};
use handjam::{gentest::*, dyn_call::{generate_linear_exec}};

fn bench_chain(c: &mut Criterion) {
    let mut state = ChainState::default();
    let mut dirty = ChainDirty::default();

    state.value0 = Some(0);

    let benchfn = move || {
        let count = chain(&mut state, &mut dirty);
        assert_eq!(count,CHAIN_LENGTH);
        assert_eq!(state.value9,Some(9));
        dirty.set_needs_compute(0);
    };

    c.bench_function("chain", |b| b.iter(benchfn));
}

fn bench_straightchain(c: &mut Criterion) {
    let mut state = ChainState::default();
    let mut dirty = ChainDirty::default();

    state.value0 = Some(0);

    let benchfn = move || {
        let count = chain_straightline(&mut state, &mut dirty);
        assert_eq!(count,CHAIN_LENGTH);
        assert_eq!(state.value9,Some(9));
        dirty.set_needs_compute(0);
    };

    c.bench_function("chain_straightline", |b| b.iter(benchfn));
}

fn bench_dynamic(c: &mut Criterion) {
   let mut exec = generate_linear_exec(CHAIN_LENGTH);

    c.bench_function("chain_dynamic", |b| b.iter(|| {
        exec.set_runnable(0);
         let count = exec.run().expect("Run should succeed");
         assert_eq!(count,CHAIN_LENGTH);
         assert_eq!(exec.value::<i32>(21),Some(&21));
         assert_eq!(exec.value::<i32>(9).unwrap(),&9);
         assert_eq!(exec.value::<i32>(99).unwrap(),&99);
    }));
}


criterion_group!(benches, bench_chain, bench_straightchain, bench_dynamic);
criterion_main!(benches);

