#![allow(dead_code)]

use handjam::*;

// we're going to generate this picture/
// three -> mutable_int -> tick -> int_to_string -> string_slice -> print
fn manual() -> String {
    let three_value = three();
    let mut mut_int = mutable_int(three_value);
    let tick_value = tick(&mut mut_int);
    let int_to_string_value = int_to_string(tick_value);
    let string_slice_value = string_slice(&int_to_string_value);
    print(string_slice_value);
    string_slice_value.into()
}

#[derive(Default)]
struct ManualPersist {
    three_value: i32,
    mut_int: MutInt,
    tick_value: i32,
    int_to_string_value: String,
}

fn manual_persist(state: &mut ManualPersist) -> String {
    state.three_value = three();
    state.mut_int = mutable_int(state.three_value);
    state.tick_value = tick(&mut state.mut_int);
    state.int_to_string_value = int_to_string(state.tick_value);
    let string_slice_value = string_slice(&state.int_to_string_value);
    print(string_slice_value);
    string_slice_value.into()
}

#[derive(Default)]
struct ManualOptionalStore {
    three_value: Option<i32>,
    mut_int: Option<MutIntNoDefault>,
    tick_value: Option<i32>,
    int_to_string_value: Option<String>,
    print_value: Option<()>,
}

fn manual_optional_persist_with_unwrap(state: &mut ManualOptionalStore) -> String {
    state.three_value = Some(three());
    state.mut_int = Some(mutable_int_no_default(state.three_value.unwrap()));
    state.tick_value = Some(tick_no_default(&mut state.mut_int.as_mut().unwrap()));
    state.int_to_string_value = Some(int_to_string(state.tick_value.unwrap()));
    let string_slice_value = string_slice(state.int_to_string_value.as_ref().unwrap());
    print(string_slice_value);
    string_slice_value.into()
}

fn manual_option_persist_with_if_let(state: &mut ManualOptionalStore) -> String {
    state.three_value = Some(three());

    if let Some(three_value) = state.three_value {
        state.mut_int = Some(mutable_int_no_default(three_value));
    }

    if let Some(mut_int) = state.mut_int.as_mut() {
        state.tick_value = Some(tick_no_default(mut_int));
    }

    if let Some(tick_value) = state.tick_value {
        state.int_to_string_value = Some(int_to_string(tick_value));
    }

    let string_slice_value = string_slice(state.int_to_string_value.as_ref().unwrap());
    print(string_slice_value);
    string_slice_value.into()
}

#[derive(PartialEq)]
enum DirtyEnum {
    NeedCompute,
    Stale,
    Clean,
}

impl Default for DirtyEnum {
    fn default() -> Self {
        DirtyEnum::NeedCompute
    }
}

#[derive(Default)]
struct DirtyState {
    three: DirtyEnum,
    mut_int: DirtyEnum,
    tick: DirtyEnum,
    int_to_string: DirtyEnum,
    print: DirtyEnum,
}

fn manual_dirty(state: &mut ManualOptionalStore, dirty: &mut DirtyState) {
    if dirty.three == DirtyEnum::NeedCompute {
        // No dependencies to check, but we'll stay with the form
        state.three_value = {
            dirty.three = DirtyEnum::Clean;
            Some(three())
        }; // no else, it just works
           // dirty children
        dirty.mut_int = DirtyEnum::NeedCompute;
    }

    if dirty.mut_int == DirtyEnum::NeedCompute {
        state.mut_int = if let Some(three_value) = state.three_value {
            dirty.mut_int = DirtyEnum::Clean;
            Some(mutable_int_no_default(three_value))
        } else {
            dirty.mut_int = DirtyEnum::Stale;
            None
        };
        dirty.tick = DirtyEnum::NeedCompute;
    }

    if dirty.tick == DirtyEnum::NeedCompute {
        state.tick_value = if let Some(mut_int) = state.mut_int.as_mut() {
            dirty.tick = DirtyEnum::Clean;
            Some(tick_no_default(mut_int))
        } else {
            dirty.tick = DirtyEnum::Stale;
            None
        };
        dirty.int_to_string = DirtyEnum::NeedCompute;
    }

    if dirty.int_to_string == DirtyEnum::NeedCompute {
        state.int_to_string_value = if let Some(tick_value) = state.tick_value {
            dirty.int_to_string = DirtyEnum::Clean;
            Some(int_to_string(tick_value))
        } else {
            dirty.int_to_string = DirtyEnum::Stale;
            None
        };
        dirty.print = DirtyEnum::NeedCompute;
    }

    if dirty.print == DirtyEnum::NeedCompute {
        state.print_value = if let Some(int_to_string_value) = state.int_to_string_value.as_ref() {
            // Again, we have to compute this slice inside here because we don't know if the dependencies are valid
            let string_slice_value = string_slice(int_to_string_value);
            print(string_slice_value);
            dirty.print = DirtyEnum::Clean;
            Some(())
        } else {
            dirty.print = DirtyEnum::Stale;
            None
        }
    }
}

// Do a tree
// one
//  \
//   \----add
//   /
//  /
// two
#[derive(Default)]
struct TreeDirty {
    one: DirtyEnum,
    two: DirtyEnum,
    add: DirtyEnum,
}

#[derive(Default)]
struct TreeState {
    one: Option<i32>,
    two: Option<i32>,
    add: Option<i32>,
}

fn manual_tree(state: &mut TreeState, dirty: &mut TreeDirty) {
    if dirty.one == DirtyEnum::NeedCompute {
        state.one = {
            dirty.one = DirtyEnum::Clean;
            Some(one())
        };
        dirty.add = DirtyEnum::NeedCompute;
    }

    if dirty.two == DirtyEnum::NeedCompute {
        state.two = {
            dirty.two = DirtyEnum::Clean;
            Some(two())
        };
        dirty.add = DirtyEnum::NeedCompute;
    }

    if dirty.add == DirtyEnum::NeedCompute {
        state.add = if let (Some(one), Some(two)) = (state.one, state.two) {
            dirty.add = DirtyEnum::Clean;
            Some(add(one, two))
        } else {
            dirty.add = DirtyEnum::Stale;
            None
        }
    }
}

#[derive(Default)]
struct TreeStateWithOptional {
    one: Option<i32>,
    two: Option<Option<i32>>,
    add: Option<i32>,
}

fn tree_with_optional_node(state: &mut TreeStateWithOptional, dirty: &mut TreeDirty) {
    if dirty.one == DirtyEnum::NeedCompute {
        state.one = {
            dirty.one = DirtyEnum::Clean;
            Some(one())
        };
        dirty.add = DirtyEnum::NeedCompute;
    }

    if dirty.two == DirtyEnum::NeedCompute {
        state.two = {
            dirty.two = DirtyEnum::Clean;
            Some(two_optional())
        };
        dirty.add = DirtyEnum::NeedCompute;
    }

    if dirty.add == DirtyEnum::NeedCompute {
        state.add = if let (Some(one), Some(Some(two))) = (state.one, state.two) {
            dirty.add = DirtyEnum::Clean;
            Some(add(one, two))
        } else {
            dirty.add = DirtyEnum::Stale;
            None
        }
    }
}

struct TreeStateWithResult {
    one: Option<Result<i32, ()>>,
    two: Option<i32>,
    add: Option<i32>,
}

fn tree_with_result_node(state: &mut TreeStateWithResult, dirty: &mut TreeDirty) {
    if dirty.one == DirtyEnum::NeedCompute {
        state.one = {
            dirty.one = DirtyEnum::Clean;
            Some(one_result())
        };
        dirty.add = DirtyEnum::NeedCompute;
    }

    if dirty.two == DirtyEnum::NeedCompute {
        state.two = {
            dirty.two = DirtyEnum::Clean;
            Some(two())
        };
        dirty.add = DirtyEnum::NeedCompute;
    }

    if dirty.add == DirtyEnum::NeedCompute {
        state.add = if let (Some(Ok(one)), Some(two)) = (state.one, state.two) {
            dirty.add = DirtyEnum::Clean;
            Some(add(one, two))
        } else {
            dirty.add = DirtyEnum::Stale;
            None
        }
    }

    // What's the generic of this look like?

    // COMPUTE_STATE = HAS_INPUTS_COMPUTE | NO_INPUTS_COMPUTE
    // HAS_INPUTS_COMPUTE = IF_GET_INPUTS {
    //   COMPUTE_STATE = CLEAN;
    //   Some(run_node(INPUTS))
    // } ELSE {
    //   COMPUTE_STATE = STALE;
    //   None
    // }
    // NO_INPUTS_COMPUTE = {
    //   COMPUTE_STATE = CLEAN;
    //   Some(run_node())
    // }

    // if THIS_NODE_NEEDS_COMPUTE {
    //     NODE_STATE = {
    //       IF_GET_INPUTS {
    //       COMPUTE_STATE = CLEAN;
    //       Some(run_node(INPUTS))
    //     } OPTIONAL_ELSE {
    //       COMPUTE_STATE = STALE;
    //       None
    //     }
    // }
}

fn tree_with_macros(state: &mut TreeState, dirty: &mut TreeDirty) {
    run_operation!(
        runstate: dirty.one,
        output: state.one,
        function: one,
        children: dirty.add
    );

    run_operation!(
        runstate: dirty.two,
        output: state.two,
        function: two,
        children: dirty.add
    );

    run_operation!(
        runstate: dirty.add,
        output: state.add,
        inputs: (one,two) = (state.one, state.two),
        function: add,
        children:
    );
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual() {
        let res = manual();
        assert_eq!(res, "4");
    }

    #[test]
    fn test_manual_persist() {
        let mut state = ManualPersist::default();
        let res = manual_persist(&mut state);
        assert_eq!(res, "4");
    }

    #[test]
    fn test_manual_optional_persist_with_unwrap() {
        let mut state = ManualOptionalStore::default();
        let res = manual_optional_persist_with_unwrap(&mut state);
        assert_eq!(res, "4");
    }

    #[test]
    fn test_manual_option_persist_with_if_let() {
        let mut state = ManualOptionalStore::default();
        let res = manual_option_persist_with_if_let(&mut state);
        assert_eq!(res, "4");
    }

    #[test]
    fn test_manual_dirty() {
        let mut state = ManualOptionalStore::default();
        let mut dirty = DirtyState::default();
        manual_dirty(&mut state, &mut dirty);

        assert_eq!(state.int_to_string_value, Some("4".into()));
    }

    #[test]
    fn test_manual_tree() {
        let mut state = TreeState::default();
        let mut dirty = TreeDirty::default();
        manual_tree(&mut state, &mut dirty);

        assert_eq!(state.add, Some(3));
    }

    #[test]
    fn test_tree_with_optional_node() {
        let mut state = TreeStateWithOptional::default();
        let mut dirty = TreeDirty::default();
        tree_with_optional_node(&mut state, &mut dirty);

        assert_eq!(state.add, Some(3));
    }

    #[test]
    fn test_tree_with_macros() {
        let mut state = TreeState::default();
        let mut dirty = TreeDirty::default();
        tree_with_macros(&mut state, &mut dirty);

        assert_eq!(state.add, Some(3));
    }
}
