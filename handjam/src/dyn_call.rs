use std::rc::Rc;

use ive::make_dynamicable;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DirtyEnum {
    NeedCompute,
    Stale,
    Clean,
}

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

pub trait ValidInputs {
    fn get<'a, T>(&self, index: usize) -> &'a T;
}

//pub type BoxedAny = Box<dyn std::any::Any>;
pub struct BoxedAny {
    any: Box<dyn std::any::Any>,
}
impl BoxedAny {
    fn new<T>(value: T) -> BoxedAny
    where
        T: 'static + std::any::Any,
    {
        Self {
            any: Box::new(value),
        }
    }

    pub fn value<T>(&self) -> &T
    where
        T: 'static + std::any::Any,
    {
        self.any
            .downcast_ref::<T>()
            .expect("Unable to downcast any to given type")
    }
}

pub type OptionalValue = Option<BoxedAny>;
pub type AnyInputs<'a> = [&'a BoxedAny];
pub type AnyOutputs<'a> = [OptionalValue];
type DynCallResult = Result<(), Box<dyn std::error::Error>>;
pub trait DynCall {
    fn call(&self, inputs: ArrayGather, outputs: &mut ArrayScatter) -> DynCallResult;
    fn input_len(&self) -> usize;
    fn output_len(&self) -> usize;
}

#[make_dynamicable()]
fn zero() -> i32 {
    0
}

pub struct DynStorage {
    values: Vec<Option<BoxedAny>>,
}
impl DynStorage {
    fn new(size: usize) -> Self {
        let mut s = DynStorage {
            values: Vec::with_capacity(size),
        };
        for _ in 0..size {
            s.values.push(None);
        }
        s
    }
}

pub struct DynDirty {
    state: Vec<DirtyEnum>,
}

impl DynDirty {
    fn new(size: usize) -> Self {
        let mut s = DynDirty {
            state: Vec::with_capacity(size),
        };
        for _ in 0..size {
            s.state.push(DirtyEnum::NeedCompute);
        }
        s
    }
}

type ChildrenIndices = Vec<usize>;
type InputIndices = Vec<usize>;
struct ExecNode {
    call: Box<dyn DynCall>,
    input_indices: InputIndices,
    children: ChildrenIndices,
}
impl ExecNode {
    fn num_inputs(&self) -> usize {
        self.input_indices.len()
    }
    fn num_outputs(&self) -> usize {
        self.call.output_len()
    }
}
pub struct DynLinearExec {
    store: DynStorage,
    dirty: DynDirty,
    nodes: Vec<ExecNode>,
}

// Create a DynExecError that is an std::error::Error
#[derive(Debug, PartialEq)]
enum DynExecError {
    DevBadDirtyIndex,
    DevInputOutOfRange,
}
impl std::fmt::Display for DynExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DynExecError::DevBadDirtyIndex => write!(f, "Dev Error: Bad dirty index"),
            DynExecError::DevInputOutOfRange => write!(f, "Dev Error: Input out of range"),
        }
    }
}
impl std::error::Error for DynExecError {
    fn description(&self) -> &str {
        match *self {
            DynExecError::DevBadDirtyIndex => "Dev Error: Bad dirty index",
            DynExecError::DevInputOutOfRange => "Dev Error: Input out of range",
        }
    }
}

trait InputFetch {
    fn fetch<T>(&self, index: usize) -> &T
    where
        T: 'static + std::any::Any;
    fn len() -> usize;
}
pub struct ArrayGather<'a> {
    values: &'a [OptionalValue],
    indices: &'a [usize],
}
impl<'a> ArrayGather<'a> {
    pub fn fetch<T>(&'a self, index: usize) -> &'a T
    where
        T: 'static + std::any::Any,
    {
        self.values[self.indices[index]]
            .as_ref()
            .unwrap()
            .value::<T>()
    }
    pub fn len(&self) -> usize {
        self.indices.len()
    }
}
pub struct ArrayScatter<'a> {
    values: &'a mut [OptionalValue]
}
impl<'a> ArrayScatter<'a> {
    pub fn some<T>(&mut self, index: usize, value: T)
    where
        T: 'static + std::any::Any,
    {
        self.values[index] = Some(BoxedAny::new(value));
    }
    pub fn none(&mut self, index: usize)
    {
        self.values[index] = None;
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl DynLinearExec {
    fn new(nodes: impl Iterator<Item = Box<dyn DynCall>>) -> Self {
        let nodes = nodes
            .map(|n| ExecNode {
                call: n,
                input_indices: Vec::new(),
                children: Vec::new(),
            })
            .collect::<Vec<_>>();

        let size = nodes.len();
        let storesize = nodes.iter().map(|n| n.num_outputs()).sum();
        Self {
            store: DynStorage::new(storesize),
            dirty: DynDirty::new(size),
            nodes,
        }
    }
    pub fn new_linear_chain(nodes: impl Iterator<Item = Box<dyn DynCall>>) -> Self {
        let mut exec = Self::new(nodes);
        for i in 1..exec.nodes.len() {
            exec.inputs(i, vec![i - 1]);
            exec.children(i - 1, vec![i]);
        }
        exec
    }
    pub fn is_some(&self, index: usize) -> bool {
        self.store.values[index].is_some()
    }
    pub fn is_none(&self, index: usize) -> bool {
        self.store.values[index].is_none()
    }
    pub fn value_any(&self, index: usize) -> &Option<BoxedAny> {
        &self.store.values[index]
    }
    pub fn value<T>(&self, index: usize) -> Option<&T>
    where
        T: 'static + std::any::Any,
    {
        let v = self.value_any(index);
        let v = v.as_ref();
        if let Some(v) = v {
            Some(v.value())
        } else {
            None
        }
    }

    pub fn run_state(&self, index: usize) -> DirtyEnum {
        self.dirty.state[index]
    }
    pub fn set_runnable(&mut self, index: usize) {
        self.dirty.state[index] = DirtyEnum::NeedCompute;
    }
    pub fn run(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let nodes = &self.nodes;
        let dirty = &mut self.dirty;
        let store = &mut self.store;

        let mut compute_count = 0usize;

        // The nodes store their outputs in order.  This keeps track of the index of the next output
        let mut output_index = 0;
        for (run_index, node) in nodes.iter().enumerate() {
            let runstate = dirty
                .state
                .get_mut(run_index)
                .ok_or(DynExecError::DevBadDirtyIndex)?;

            // As much as I lothe nested indentation, I want to keep the same format as the "algorithm"
            if *runstate == DirtyEnum::NeedCompute {
                assert_eq!(
                    node.num_inputs(),
                    node.input_indices.len(),
                    "Input indices not set correctly"
                );

                // By definition, the inputs must be earlier in the store
                // than the outputs.  Split the store into two slices, one
                // for inputs and one for outputs.  We do this so we can
                // borrow the inputs and outputs separately.
                // The output_index is where that break happens
                let (inputs, outputs) = store.values.split_at_mut(output_index);

                // Do a quick sanity check that all the input indicies requested
                // are in range
                {
                    let max_input_index = inputs.len();
                    if node.input_indices.iter().any(|i| *i >= max_input_index) {
                        return Err(DynExecError::DevInputOutOfRange.into());
                    }
                }

                // See if any of the inputs are None
                let missing_inputs = node.input_indices.iter().any(|i| inputs[*i].is_none());

                // We only need our specific output range.
                let outputs = &mut outputs[0..node.num_outputs()];

                if !missing_inputs {
                    *runstate = DirtyEnum::Clean;

                    let fetch = ArrayGather {
                        values: inputs,
                        indices: &node.input_indices,
                    };

                    let mut setter = ArrayScatter {
                        values: outputs
                    };

                    node.call.call(fetch, &mut setter)?;

                    compute_count += 1;
                } else {
                    dirty.state[run_index] = DirtyEnum::Stale;
                    // This slick one liner sets all the outputs to None
                    outputs.into_iter().for_each(|o| *o = None);
                }
                // Run our children
                for child in node.children.iter() {
                    dirty.state[*child] = DirtyEnum::NeedCompute;
                }
            }
            output_index += node.num_outputs();
        }
        Ok(compute_count)
    }

    fn children(&mut self, node_index: usize, children: ChildrenIndices) {
        self.nodes[node_index].children = children;
    }

    fn inputs(&mut self, node_index: usize, indices: Vec<usize>) {
        self.nodes[node_index].input_indices = indices;
    }
}

pub fn box_dyn_call<T: DynCall + 'static>(t: T) -> Box<dyn DynCall> {
    Box::new(t)
}

pub fn generate_linear_exec(count: usize) -> DynLinearExec {
    let nodes = (0..count - 1).map(|_| Box::new(AddOneDynCall {}) as Box<dyn DynCall>);

    let firstnode = vec![Box::new(ZeroDynCall {}) as Box<dyn DynCall>];

    let concat = firstnode.into_iter().chain(nodes);

    DynLinearExec::new_linear_chain(concat)
}

#[cfg(test)]
mod tests {

    use ive::make_dynamicable;

    use super::*;

    #[test]
    fn test_loop() {
        let one_compute = OneDynCall {};
        let addone_compute = AddOneDynCall {};

        let nodes: Vec<Box<dyn DynCall>> = vec![Box::new(one_compute), Box::new(addone_compute)];

        let mut exec = DynLinearExec::new_linear_chain(nodes.into_iter());

        let computed = exec.run().expect("Failed to run");

        assert_eq!(computed, 2);

        let value1 = exec.value_any(1);
        if let Some(value1) = value1 {
            let value1_box = value1.value::<i32>();
            assert_eq!(*value1_box, 2);
        } else {
            assert!(false);
        }

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
        assert_eq!(exec.value::<i32>(count - 1), Some(&2));
    }

    #[test]
    fn test_hand_made_graph() {
        let mut exec = DynLinearExec::new(
            vec![
                box_dyn_call(OneDynCall {}),
                box_dyn_call(TwoDynCall {}),
                box_dyn_call(AddDynCall {}),
            ]
            .into_iter(),
        );

        // Wire the inputs manuall
        exec.inputs(0, vec![]);
        exec.inputs(1, vec![]);
        exec.inputs(2, vec![0, 1]);

        // Wire the children manually
        exec.children(0, vec![2]);
        exec.children(1, vec![2]);
        exec.children(2, vec![]);

        let count = exec.run().expect("Failed to run");
        assert_eq!(count, 3);
        assert_eq!(exec.value::<i32>(count - 1), Some(&3));
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
}
