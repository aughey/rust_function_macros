
#[derive(Copy,Clone,PartialEq)]
pub enum DirtyEnum {
    NeedCompute,
    Stale,
    Clean,
}

pub fn one() -> i32 {
    1
}

pub fn add_one(a: i32) -> i32 {
    a + 1
}

pub trait ValidInputs {
    fn get<'a, T>(&self, index: usize) -> &'a T;
}

pub type BoxedAny = Box<dyn std::any::Any>;
pub type AnyInputs<'a> = [&'a BoxedAny];
pub trait DynCall {
    fn call(&self, inputs: &AnyInputs) -> BoxedAny;
    fn input_len(&self) -> usize;
}

fn zero() -> i32 {
    0
}

struct ZeroAsDynCall;
impl DynCall for ZeroAsDynCall {
    fn call(&self, _inputs: &AnyInputs) -> BoxedAny {
        Box::new(zero())
    }
    fn input_len(&self) -> usize {
        0
    }
}

struct OneAsDynCall {

}

impl DynCall for OneAsDynCall {
    fn call(&self, _inputs: &AnyInputs) -> BoxedAny {
        Box::new(one())
    }
    fn input_len(&self) -> usize {
        0
    }
}

struct AddOneAsDynCall;

impl DynCall for AddOneAsDynCall {
    fn call(&self, inputs: &AnyInputs) -> BoxedAny {
        let a = inputs[0].downcast_ref::<i32>().unwrap();
        Box::new(add_one(*a))
    }
    fn input_len(&self) -> usize {
        1
    }
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
pub struct LinearExec {
    store: DynStorage,
    dirty: DynDirty,
    nodes: Vec<Box<dyn DynCall>>,
    input_indices: Vec<InputIndices>,
    children: Vec<ChildrenIndices>,
}

impl LinearExec {
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
    pub fn value_any(&self, index: usize) -> Option<&BoxedAny> {
        self.store.values.get(index)?.as_ref()
    }
    pub fn value<T>(& self, index: usize) -> Option<& T> 
        where T: Copy + 'static {
        let any = self.value_any(index)?;
        any.downcast_ref::<T>()
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
                    store.values[run_index] = Some(node.call(&inputs));
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

pub fn generate_linear_exec(count: usize) -> LinearExec {
    let nodes = (0..count-1).map(|_| {
        Box::new(AddOneAsDynCall{}) as Box<dyn DynCall>
    });

    let firstnode = vec![Box::new(ZeroAsDynCall{}) as Box<dyn DynCall>];

    let concat = firstnode.into_iter().chain(nodes);

    let mut le = LinearExec::new(concat);

    for i in 0..count-1 {
        le.children(i,vec![i+1]);
        le.inputs(i+1,vec![i]);
    }

    le
}

#[cfg(test)]
mod tests {
    use super::*;
  
    #[test]
    fn test_one() {
        let one_compute = OneAsDynCall {};
        let addone_compute = AddOneAsDynCall {};

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
            let value1_box = value1.downcast_ref::<i32>().unwrap();
            assert_eq!(*value1_box, 2);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_loop() {
        let one_compute = OneAsDynCall {};
        let addone_compute = AddOneAsDynCall {};

        let nodes : Vec<Box<dyn DynCall>>= vec![Box::new(one_compute), Box::new(addone_compute)];

        let mut exec = LinearExec::new(nodes.into_iter());
        exec.children(0, vec![1]);
        exec.inputs(1, vec![0]);
        
        let computed = exec.run();

        assert_eq!(computed, 2);
    
        let value1 = exec.value_any(1);
        if let Some(value1) = value1 {
            let value1_box = value1.downcast_ref::<i32>().unwrap();
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
        assert_eq!(exec.value_any(9).unwrap().downcast_ref::<i32>(),Some(&9));
        assert_eq!(exec.value_any(9).unwrap().downcast_ref::<i32>(),Some(&9));

        exec.set_runnable(0);
        let count = exec.run();
        assert_eq!(count,CHAIN_LENGTH);
    }
}