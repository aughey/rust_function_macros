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
    Dirty,
    Clean,
}

impl Default for DirtyEnum {
    fn default() -> Self {
        DirtyEnum::Dirty
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
    if dirty.three != DirtyEnum::Clean {
        // No dependencies to check, but we'll stay with the form
        state.three_value = {
            dirty.three = DirtyEnum::Clean;
            // dirty children
            dirty.mut_int = DirtyEnum::Dirty;
            Some(three())
        } // No else, it must succeed
    }

    if dirty.mut_int != DirtyEnum::Clean {
        state.mut_int = if let Some(three_value) = state.three_value {
            dirty.mut_int = DirtyEnum::Clean;
            dirty.tick = DirtyEnum::Dirty;
            Some(mutable_int_no_default(three_value))
        } else {
            None
        }
    }

    if dirty.tick != DirtyEnum::Clean {
        state.tick_value = if let Some(mut_int) = state.mut_int.as_mut() {
            dirty.tick = DirtyEnum::Clean;
            dirty.int_to_string = DirtyEnum::Dirty;
            Some(tick_no_default(mut_int))
        } else {
            None
        }
    }

    if dirty.int_to_string != DirtyEnum::Clean {
        state.int_to_string_value = if let Some(tick_value) = state.tick_value {
            dirty.int_to_string = DirtyEnum::Clean;
            dirty.print = DirtyEnum::Dirty;
            Some(int_to_string(tick_value))
        } else {
            None
        }
    }

    if dirty.print != DirtyEnum::Clean {
        state.print_value = if let Some(int_to_string_value) = state.int_to_string_value.as_ref() {
            // Again, we have to compute this slice inside here because we don't know if the dependencies are valid
            let string_slice_value = string_slice(int_to_string_value);
            print(string_slice_value);
            dirty.print = DirtyEnum::Clean;
            Some(())
        } else {
            None
        }
    }
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
}
