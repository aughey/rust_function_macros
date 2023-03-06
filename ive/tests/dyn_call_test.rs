use std::rc::Rc;

use ive_macros::make_dynamicable;

#[make_dynamicable()]
pub fn one() -> i32 {
    1
}

#[make_dynamicable()]
pub fn two() -> i32 {
    2
}

#[make_dynamicable()]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[make_dynamicable()]
pub fn add_one(a: i32) -> i32 {
    a + 1
}
// struct AddOneDynCall;
// impl DynCall for AddOneDynCall {
//     fn call(&self, inputs: &AnyInputs, outputs: &AnyOutputs) -> DynCallResult {
//         assert_eq!(inputs.len(), 1);
//         assert_eq!(outputs.len(), 1);
//         let output = BoxedAny::new(add_one(*inputs[0].value()) );
//         let mut thisout = &outputs[0];
//        *thisout = Some(output);
//         Ok(())
//     }
//     fn input_len(&self) -> usize {
//         1
//     }
//     fn output_len(&self) -> usize {
//         1
//     }
// }

#[make_dynamicable()]
fn is_even(v: i32) -> Option<i32> {
    if v % 2 == 0 {
        Some(v)
    } else {
        None
    }
}

#[make_dynamicable()]
fn returns_error() -> Result<i32, String> {
    Err("Error".to_string())
}

#[derive(Clone)]
struct CustomType {
    value: i32,
    mutable_value: Rc<std::cell::RefCell<i32>>,
}

#[make_dynamicable()]
fn create_custom_type() -> CustomType {
    CustomType {
        value: 1,
        mutable_value: Rc::new(std::cell::RefCell::new(1)),
    }
}

#[make_dynamicable()]
fn increment_custom_type(custom_type: &CustomType) -> CustomType {
    CustomType {
        value: custom_type.value + 1,
        mutable_value: custom_type.mutable_value.clone(),
    }
}

#[make_dynamicable()]
fn increment_mutable(custom_type: &CustomType) -> CustomType {
    let mut value = custom_type.mutable_value.borrow_mut();
    *value += 1;
    custom_type.clone()
}

#[make_dynamicable()]
fn strip_custom_type(custom_type: &CustomType) -> i32 {
    custom_type.value
}

#[make_dynamicable()]
fn zero() -> i32 {
    0
}



use ive::dyn_call::{box_dyn_call, DirtyEnum, DynLinearExec};

#[test]
fn test_loop() {
    let one_compute = OneDynCall {};
    let addone_compute = AddOneDynCall {};

    let nodes = vec![box_dyn_call(one_compute), box_dyn_call(addone_compute)];

    let mut exec = DynLinearExec::new_linear_chain(nodes.into_iter());

    let computed = exec.run().expect("Failed to run");

    assert_eq!(computed, 2);

    let value1 = exec.value::<i32>(1).unwrap();
    assert_eq!(*value1, 2);

    exec.set_runnable(0);
    let computed = exec.run().expect("Failed to run");
    assert_eq!(computed, 2);
}

#[test]
fn test_dyn_chain() {
    const CHAIN_LENGTH: usize = 10;
    let mut exec = generate_linear_exec(CHAIN_LENGTH);
    let count = exec.run().expect("Failed to run");
    assert_eq!(count, CHAIN_LENGTH);
    assert_eq!(exec.value::<i32>(9).unwrap(), &9);

    exec.set_runnable(0);
    let count = exec.run().expect("failed to run");
    assert_eq!(count, CHAIN_LENGTH);
}

#[test]
fn test_dyn_string_ops() {
    #[make_dynamicable()]
    fn john_aughey() -> String {
        "John Aughey".to_string()
    }

    #[make_dynamicable()]
    fn string_double(input: &String) -> String {
        input.to_owned() + input
    }

    let mut exec = DynLinearExec::new_linear_chain(
        vec![
            box_dyn_call(JohnAugheyDynCall {}),
            box_dyn_call(StringDoubleDynCall {}),
        ]
        .into_iter(),
    );
    let count = exec.run().expect("failed to run");
    assert_eq!(count, 2);
    assert_eq!(exec.value::<String>(0).unwrap(), &"John Aughey");
    assert_eq!(exec.value::<String>(1).unwrap(), &"John AugheyJohn Aughey");
}

#[test]
fn test_custom_type() {
    let mut exec = DynLinearExec::new_linear_chain(
        vec![
            box_dyn_call(CreateCustomTypeDynCall {}),
            box_dyn_call(IncrementCustomTypeDynCall {}),
            box_dyn_call(IncrementMutableDynCall {}),
            box_dyn_call(StripCustomTypeDynCall {}),
        ]
        .into_iter(),
    );
    let count = exec.run().expect("failed to run");
    assert_eq!(count, 4);
    assert_eq!(exec.value::<i32>(count - 1).unwrap(), &2);
}

#[test]
fn test_optional_output() {
    let nodes = vec![
        box_dyn_call(OneDynCall {}),
        box_dyn_call(IsEvenDynCall {}),
        box_dyn_call(AddOneDynCall {}),
    ];

    let mut exec = DynLinearExec::new_linear_chain(nodes.into_iter());

    let count = exec.run().expect("Failed to run");
    assert_eq!(count, 2); // last one shouldn't run
    assert_eq!(exec.run_state(0), DirtyEnum::Clean);
    assert_eq!(exec.run_state(1), DirtyEnum::Clean);
    assert_eq!(exec.run_state(2), DirtyEnum::Stale);

    let count = exec.run().expect("Failed to run");
    assert_eq!(count, 0);

    // Now make a new one with an even number
    let nodes = vec![
        box_dyn_call(TwoDynCall {}),
        box_dyn_call(IsEvenDynCall {}),
        box_dyn_call(AddOneDynCall {}),
    ];

    let mut exec = DynLinearExec::new_linear_chain(nodes.into_iter());

    let count = exec.run().expect("Failed to run");
    assert_eq!(count, 3); // All runs
}

#[test]
fn test_result_output() {
    let nodes = vec![box_dyn_call(ReturnsErrorDynCall {})];

    let mut exec = DynLinearExec::new_linear_chain(nodes.into_iter());
    let count = exec.run().expect("Failed to run");
    assert_eq!(count, 1);

    assert!(exec.is_none(0));
    assert!(exec.is_some(1));

    let err = exec.value::<String>(1).unwrap();
    assert_eq!(err, "Error");
}
