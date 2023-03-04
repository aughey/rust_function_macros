use criterion::{criterion_group, criterion_main, Criterion};
use handjam::gentest::*;

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


criterion_group!(benches, bench_chain);
criterion_main!(benches);

