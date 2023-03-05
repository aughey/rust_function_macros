use std::rc::Rc;

use ive::make_dynamicable;

#[derive(Copy,Clone,PartialEq)]
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

#[derive(Clone)]
struct CustomType {
    value: i32,
    mutable_value: Rc<std::cell::RefCell<i32>>
}

#[make_dynamicable()]
fn create_custom_type() -> CustomType {
    CustomType { value: 1, mutable_value: Rc::new(std::cell::RefCell::new(1)) }
}

#[make_dynamicable()]
fn increment_custom_type(custom_type: &CustomType) -> CustomType {
    CustomType { value: custom_type.value + 1, mutable_value: custom_type.mutable_value.clone() }
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
    any: Box<dyn std::any::Any>
}
impl BoxedAny {
    fn new<T>(value: T) -> BoxedAny 
    where T: 'static + std::any::Any
    {
        Self {
            any: Box::new(value)
        }
    }

    pub fn value<T>(&self) -> &T
    where T: 'static + std::any::Any
     {
        self.any.downcast_ref::<T>().expect("Unable to downcast any to given type")
    }
}

pub type AnyInputs<'a> = [&'a BoxedAny];
pub type AnyOutputs<'a> = [&'a mut Option<BoxedAny>];
type DynCallResult = Result<(), Box<dyn std::error::Error>>;
pub trait DynCall {
    fn call(&self, inputs: &AnyInputs, outputs: &AnyOutputs) -> DynCallResult;
    fn input_len(&self) -> usize;
    fn output_len(&self) -> usize;
}

#[make_dynamicable()]
fn zero() -> i32 {
    0
}

pub struct DynStorage {
    values: Vec<Option<BoxedAny>>
}
impl DynStorage {
    fn new(size: usize) -> Self {
        let mut s = DynStorage {
            values: Vec::with_capacity(size)
        };
        for _ in 0..size {
            s.values.push(None);
        }
        s
    }
}

pub struct DynDirty {
    state: Vec<DirtyEnum>
}

impl DynDirty {
    fn new(size: usize) -> Self {
        let mut s = DynDirty {
            state: Vec::with_capacity(size)
        };
        for _ in 0..size {
            s.state.push(DirtyEnum::NeedCompute);
        }
        s
    }
}

type ChildrenIndices = Vec<usize>;
type InputIndices = Vec<usize>;
pub struct DynLinearExec {
    store: DynStorage,
    dirty: DynDirty,
    nodes: Vec<Box<dyn DynCall>>,
    input_indices: Vec<InputIndices>,
    children: Vec<ChildrenIndices>,
}

impl DynLinearExec {
    fn new(nodes: impl Iterator<Item = Box<dyn DynCall>>) -> Self {
        let nodes = nodes.collect::<Vec<_>>();
        let size = nodes.len();
        Self {
            store: DynStorage::new(size),
            dirty: DynDirty::new(size),
            nodes,
            input_indices: vec![Vec::new(); size],
            children: vec![Vec::new(); size]
        }
    }
    pub fn new_linear_chain(nodes: impl Iterator<Item = Box<dyn DynCall>>) -> Self {
       let mut exec = Self::new(nodes);
         for i in 1..exec.nodes.len() {
            exec.inputs(i, vec![i - 1]);
            exec.children(i - 1,vec![i]);
         }
         exec
    }
    pub fn value_any(&self, index: usize) -> Option<&BoxedAny> {
        self.store.values.get(index)?.as_ref()
    }
    pub fn value<T>(& self, index: usize) -> Option<& T> 
        where T: Copy + 'static {
       let v = self.store.values.get(index)?;
       let v = v.as_ref()?;
         Some(v.value())
    }

    pub fn run_state(&self, index: usize) -> DirtyEnum {
        self.dirty.state[index]
    }
    pub fn set_runnable(&mut self, index: usize) {
        self.dirty.state[index] = DirtyEnum::NeedCompute;
    }
    pub fn run(&mut self) -> usize {
        let nodes = &self.nodes;
        let  dirty = &mut self.dirty;
        let  store = &mut self.store;

        let mut running_input_index = 0;
        let mut compute_count = 0usize;
        for (run_index,node) in nodes.iter().enumerate() {
            
            if dirty.state[run_index] == DirtyEnum::NeedCompute {
                let mut inputs = Vec::<&BoxedAny>::new();
                let inputlen = node.input_len();
                assert_eq!(inputlen, self.input_indices[run_index].len(), "Input indices not set correctly");

                for _inputindex in 0..inputlen {
                    if let Some(value) = store.values[running_input_index].as_ref() {
                        inputs.push(value);
                    }
                    running_input_index += 1;
                }

                if inputs.len() == inputlen {
                    dirty.state[run_index] = DirtyEnum::Clean;
                    let output = &mut store.values[run_index];
                    let outputs : vec![output];
                    _ = Some(node.call(&inputs, outputs));
                    compute_count += 1;
                } else {
                    dirty.state[run_index] = DirtyEnum::Stale;
                }
                // Run our children
                for child in self.children[run_index].iter() {
                    dirty.state[*child] = DirtyEnum::NeedCompute;
                }
            }
        }
        compute_count
    }

    fn children(&mut self, index: usize, children: ChildrenIndices) {
        self.children[index] = children;
    }

    fn inputs(&mut self, i: usize, indices: Vec<usize>) {
        self.input_indices[i] = indices;
    }
}

pub fn box_dyn_call<T: DynCall + 'static>(t: T) -> Box<dyn DynCall> {
    Box::new(t)
}

pub fn generate_linear_exec(count: usize) -> DynLinearExec {
    let nodes = (0..count-1).map(|_| {
        Box::new(AddOneDynCall{}) as Box<dyn DynCall>
    });

    let firstnode = vec![Box::new(ZeroDynCall{}) as Box<dyn DynCall>];

    let concat = firstnode.into_iter().chain(nodes);

    DynLinearExec::new_linear_chain(concat)
}

#[cfg(test)]
mod tests {


    use ive::make_dynamicable;

    use super::*;
  
    #[test]
    fn test_one() {
        let one_compute = OneDynCall {};
        let addone_compute = AddOneDynCall {};

        let mut store = DynStorage {
            values: vec![None, None]
        };

        let mut dirty = DynDirty {
            state: vec![DirtyEnum::NeedCompute, DirtyEnum::NeedCompute]
        };

        if dirty.state[0] == DirtyEnum::NeedCompute {
            let inputs = Vec::<&BoxedAny>::new();
            if inputs.len() == 0 {
                dirty.state[0] = DirtyEnum::Clean;
                store.values[0] = Some(one_compute.call(&inputs));
            } else {
                dirty.state[0] = DirtyEnum::Stale;
            }
        }

        if dirty.state[1] == DirtyEnum::NeedCompute {
            dirty.state[1] = DirtyEnum::Clean;
            let mut inputs = Vec::<&BoxedAny>::new();
            if let Some(value) = store.values[0].as_ref() {
                inputs.push(value);
            }
            if inputs.len() == 1 {
                dirty.state[0] = DirtyEnum::Clean;
                store.values[1] = Some(addone_compute.call(&inputs));
            } else {
                dirty.state[0] = DirtyEnum::Stale;
            }
        }

        let value1 = store.values[1].as_ref();
        if let Some(value1) = value1 {
            let value1_box = value1.value::<i32>();
            assert_eq!(*value1_box, 2);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_loop() {
        let one_compute = OneDynCall {};
        let addone_compute = AddOneDynCall {};

        let nodes : Vec<Box<dyn DynCall>>= vec![Box::new(one_compute), Box::new(addone_compute)];

        let mut exec = DynLinearExec::new_linear_chain(nodes.into_iter());
        
        let computed = exec.run();

        assert_eq!(computed, 2);
    
        let value1 = exec.value_any(1);
        if let Some(value1) = value1 {
            let value1_box = value1.value::<i32>();
            assert_eq!(*value1_box, 2);
        } else {
            assert!(false);
        }

        exec.set_runnable(0);
        let computed = exec.run();
        assert_eq!(computed, 2);
        
    }

    #[test]
    fn test_dyn_chain() {
        const CHAIN_LENGTH: usize = 10;
        let mut exec = generate_linear_exec(CHAIN_LENGTH);
        let count = exec.run();
        assert_eq!(count,CHAIN_LENGTH);
        assert_eq!(exec.value_any(9).unwrap().value::<i32>(),&9);
        assert_eq!(exec.value_any(9).unwrap().value::<i32>(),&9);

        exec.set_runnable(0);
        let count = exec.run();
        assert_eq!(count,CHAIN_LENGTH);
    }

    #[test]
    fn test_dyn_string_ops() {

        #[make_dynamicable()]
        fn john_aughey() -> String{
            "John Aughey".to_string()
        }

        #[make_dynamicable()]
        fn string_double(input: &String) -> String {
            input.to_owned() + input
        }

        let mut exec = DynLinearExec::new_linear_chain(vec![
            box_dyn_call(JohnAugheyDynCall{}),
            box_dyn_call(StringDoubleDynCall{}),
        ].into_iter());
        let count = exec.run();
        assert_eq!(count,2);
        assert_eq!(exec.value_any(0).unwrap().value::<String>(),&"John Aughey");
        assert_eq!(exec.value_any(1).unwrap().value::<String>(),&"John AugheyJohn Aughey");

    }

    #[test]
    fn test_custom_type() {
      
        let mut exec = DynLinearExec::new_linear_chain(vec![
            box_dyn_call(CreateCustomTypeDynCall{}),
            box_dyn_call(IncrementCustomTypeDynCall{}),
            box_dyn_call(IncrementMutableDynCall{}),
            box_dyn_call(StripCustomTypeDynCall{}),
        ].into_iter());
        let count = exec.run();
        assert_eq!(count,4);
        assert_eq!(exec.value::<i32>(count-1),Some(&2));
    }

    #[test]
    fn test_hand_made_graph() {
      
        let mut exec = DynLinearExec::new(vec![
            box_dyn_call(OneDynCall{}),
            box_dyn_call(TwoDynCall{}),
            box_dyn_call(AddDynCall{})
        ].into_iter());

        // Wire the inputs manuall
        exec.inputs(0, vec![]);
        exec.inputs(1, vec![]);
        exec.inputs(2, vec![0,1]);

        // Wire the children manually
        exec.children(0, vec![2]);
        exec.children(1, vec![2]);
        exec.children(2, vec![]);

        let count = exec.run();
        assert_eq!(count,3);
        assert_eq!(exec.value::<i32>(count-1),Some(&3));
    }
}