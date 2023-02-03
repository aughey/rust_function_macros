use app::{operations, run_operation, exec};

fn main() {
    println!("Hello, world!");
    let mut execstate = ExecututionState::default();
    exec(&mut execstate);
    println!("store: {:?}", execstate);
}

// Create the graph:
// two
// | \
// |  \
// |   \-> add_double -> multiple_double
// |  /                /
// | /                /
// three-------------/
fn exec(execstate: &mut ExecututionState) -> u32 {
    let mut store = &mut execstate.store;
    let runnable = &mut execstate.runnable.runnable;
    let mut run_count = 0;

    run_operation!(runnable, 0, store.two_out, operations::two(), run_count, 2);
    run_operation!(
        runnable,
        1,
        store.three_out,
        operations::three(),
        run_count,
        2,
        3
    );
    run_operation!(
        runnable,
        2,
        store.add_double_out,
        operations::add_double(store.two_out, store.three_out),
        run_count,
        3
    );
    run_operation!(
        runnable,
        3,
        store.multiple_double_out,
        operations::multiple_double(store.add_double_out, store.three_out),
        run_count
    );

    run_count
}


#[test]
fn test_exec() {
    let mut execstate = ExecututionState::default();
    let run_count = exec(&mut execstate);

    assert_eq!(run_count, 4);
    assert_eq!(execstate.store.two_out, 2.0);
    assert_eq!(execstate.store.three_out, 3.0);
    assert_eq!(execstate.store.add_double_out, 5.0);
    assert_eq!(execstate.store.multiple_double_out, 15.0);

    let run_count = exec(&mut execstate);
    assert_eq!(run_count, 0);

    execstate.runnable.runnable[0] = true;
    let run_count = exec(&mut execstate);
    assert_eq!(run_count, 3);
}

#[derive(Debug, Default)]
struct ExecututionState {
    store: GraphStore,
    runnable: exec::RunnableState<4>,
}

#[derive(Debug, Default)]
struct GraphStore {
    two_out: f64,
    three_out: f64,
    add_double_out: f64,
    multiple_double_out: f64,
}
