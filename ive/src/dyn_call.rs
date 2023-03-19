#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DirtyEnum {
    NeedCompute,
    Stale,
    Clean,
}

//pub type BoxedAny = Box<dyn std::any::Any>;
pub struct BoxedAny {
    any: Box<dyn std::any::Any>,
}
impl BoxedAny {
    pub fn new<T>(value: T) -> BoxedAny
    where
        T: 'static + std::any::Any,
    {
        Self {
            any: Box::new(value),
        }
    }

    pub fn value<T>(&self) -> Result<&T, Box<dyn std::error::Error>>
    where
        T: 'static + std::any::Any,
    {
        self.any
            .downcast_ref::<T>()
            .ok_or_else(|| "Unable to downcast any to given type".into())
    }
}

pub type OptionalValue = Option<BoxedAny>;
pub type AnyInputs<'a> = [&'a BoxedAny];
pub type AnyOutputs<'a> = [OptionalValue];
pub type DynCallResult = Result<(), Box<dyn std::error::Error>>;
pub trait DynCall {
    fn call(&self, inputs: &InputGetter, outputs: &mut OutputSetter) -> DynCallResult;
    fn kind(&self) -> &'static str;
    fn input_len(&self) -> usize;
    fn output_len(&self) -> usize;
    fn inputs(&self) -> Vec<DynPort>;
    fn output_type(&self) -> &'static [&'static str];
}

pub type DynType = Vec<String>;
pub struct DynPort {
    pub name: &'static str,
    pub kind: DynType,
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
        assert_eq!(
            self.call.input_len(),
            self.input_indices.len(),
            "Dev Error: Node input length mismatch"
        );
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
    BadDirtyIndex,
    InputOutOfRange,
    FetchNone,
    ValueIsNone,
}
impl std::fmt::Display for DynExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DynExecError::BadDirtyIndex => write!(f, "Dev Error: Bad dirty index"),
            DynExecError::InputOutOfRange => write!(f, "Dev Error: Input out of range"),
            DynExecError::FetchNone => write!(f, "Dev Error: Fetch None"),
            DynExecError::ValueIsNone => write!(f, "Dev Error: Value is None"),
        }
    }
}
impl std::error::Error for DynExecError {}
// impl std::error::Error for DynExecError {
//     fn description(&self) -> &str {
//         match *self {
//             DynExecError::DevBadDirtyIndex => "Dev Error: Bad dirty index",
//             DynExecError::DevInputOutOfRange => "Dev Error: Input out of range",
//             DynExecError::DevFetchNone => "Dev Error: Fetch None",
//         }
//     }
// }

trait InputFetch {
    fn fetch<T>(&self, index: usize) -> &T
    where
        T: 'static + std::any::Any;
    fn len() -> usize;
}
pub struct InputGetter<'a> {
    values: &'a [OptionalValue],
    indices: &'a [usize],
}
impl<'a> InputGetter<'a> {
    pub fn new(values: &'a [OptionalValue], indices: &'a [usize]) -> Self {
        Self { values, indices }
    }
    pub fn fetch<T>(&'a self, index: usize) -> Result<&'a T, Box<dyn std::error::Error>>
    where
        T: 'static + std::any::Any,
    {
        let value = &self.values[self.indices[index]];
        let value = value.as_ref();
        let value = value.ok_or(DynExecError::FetchNone)?;
        //.unwrap()
        let value = value.value::<T>();
        value
    }
    pub fn len(&self) -> usize {
        self.indices.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
pub struct OutputSetter<'a> {
    values: &'a mut [OptionalValue],
    set_count: usize,
}
// impl Drop for OutputSetter<'_> {
//     fn drop(&mut self) {
//         assert_eq!(
//             self.set_count,
//             self.values.len(),
//             "Not all outputs were set"
//         );
//     }
// }
impl<'a> OutputSetter<'a> {
    pub fn new(values: &'a mut [OptionalValue]) -> Self {
        Self {
            values,
            set_count: 0,
        }
    }
    pub fn some<T>(&mut self, index: usize, value: T)
    where
        T: 'static + std::any::Any,
    {
        self.values[index] = Some(BoxedAny::new(value));
        self.set_count += 1;
    }
    pub fn none(&mut self, index: usize) {
        self.values[index] = None;
        self.set_count += 1;
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait HasDynCall {
    fn dyn_call(&self) -> Box<dyn DynCall>;
}
pub trait HasInputIndices {
    type IntoIter: Iterator<Item = usize>;
    fn input_indices(&self) -> Self::IntoIter;
}
pub trait HasChildrenIndices {
    type IntoIter: Iterator<Item = usize>;
    fn children_indices(&self) -> Self::IntoIter;
}

impl DynLinearExec {
    pub fn new(nodes: impl Iterator<Item = Box<dyn DynCall>>) -> Self {
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
    pub fn build_execution_chain<DESC, DESCITEM>(desc: DESC) -> DynLinearExec
    where
        DESC: Iterator<Item = DESCITEM>,
        DESCITEM: HasDynCall + HasInputIndices + HasChildrenIndices,
    {
        let nodes = desc
            .map(|n| ExecNode {
                call: n.dyn_call(),
                input_indices: n.input_indices().collect(),
                children: n.children_indices().collect(),
            })
            .collect::<Vec<_>>();
        let storelen = nodes.iter().map(|n| n.num_outputs()).sum();
        Self {
            store: DynStorage::new(storelen),
            dirty: DynDirty::new(nodes.len()),
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

    pub fn value<T>(&self, index: usize) -> Result<&T, Box<dyn std::error::Error>>
    where
        T: 'static + std::any::Any,
    {
        let v = self
            .store
            .values
            .get(index)
            .ok_or(DynExecError::InputOutOfRange)?;
        v.as_ref().ok_or(DynExecError::ValueIsNone)?.value::<T>()
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
                .ok_or(DynExecError::BadDirtyIndex)?;

            // As much as I lothe nested indentation, I want to keep the same format as the "algorithm"
            if *runstate == DirtyEnum::NeedCompute {
                {
                    let num_in = node.num_inputs();
                    let input_indicides = node.input_indices.len();
                    assert_eq!(num_in, input_indicides, "Input indices not set correctly");
                }

                // By definition, the inputs must be earlier in the store
                // than the outputs.  Split the store into two slices, one
                // for inputs and one for outputs.  We do this so we can
                // borrow the inputs and outputs separately.
                // The output_index is where that break happens
                let (inputs, outputs) = {
                    let (i, o) = store.values.split_at_mut(output_index);
                    // Downgrade our inputs to readonly
                    // We only need our specific output range.
                    (&i[0..i.len()], &mut o[0..node.num_outputs()])
                };

                // Do a quick sanity check that all the input indicies requested
                // are in range
                {
                    let max_input_index = inputs.len();
                    if node.input_indices.iter().any(|i| *i >= max_input_index) {
                        return Err(DynExecError::InputOutOfRange.into());
                    }
                }

                // See if any of the inputs are None
                let missing_inputs = node.input_indices.iter().any(|i| inputs[*i].is_none());

                if !missing_inputs {
                    *runstate = DirtyEnum::Clean;

                    let fetch = InputGetter {
                        values: inputs,
                        indices: &node.input_indices,
                    };

                    let mut setter = OutputSetter {
                        values: outputs,
                        set_count: 0,
                    };

                  //  let current_kind = node.call.kind();

                    node.call.call(&fetch, &mut setter)?;

                    compute_count += 1;
                } else {
                    dirty.state[run_index] = DirtyEnum::Stale;
                    // This slick one liner sets all the outputs to None
                    outputs.iter_mut().for_each(|o| *o = None);
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

    pub fn children(&mut self, node_index: usize, children: ChildrenIndices) {
        self.nodes[node_index].children = children;
    }

    pub fn inputs(&mut self, node_index: usize, indices: Vec<usize>) {
        self.nodes[node_index].input_indices = indices;
    }
}

pub fn box_dyn_call<T: DynCall + 'static>(t: T) -> Box<dyn DynCall> {
    Box::new(t)
}
