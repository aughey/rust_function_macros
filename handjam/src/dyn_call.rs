
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

pub struct LinearExec {
    store: DynStorage,
    dirty: DynDirty,
    nodes: Vec<Box<dyn DynCall>>,
}

impl LinearExec {
    fn new(nodes: impl Iterator<Item = Box<dyn DynCall>>) -> Self {
        let nodes = nodes.collect::<Vec<_>>();
        let size = nodes.len();
        Self {
            store: DynStorage::new(size),
            dirty: DynDirty::new(size),
            nodes: nodes
        }
    }
    pub fn value(&self, index: usize) -> Option<&BoxedAny> {
        self.store.values[index].as_ref()
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
        let node_count = nodes.len();
        for (run_index,node) in nodes.iter().enumerate() {
            
            if dirty.state[run_index] == DirtyEnum::NeedCompute {
                let mut inputs = Vec::<&BoxedAny>::new();
                let inputlen = node.input_len();
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
                if run_index+1 < node_count {
                    dirty.state[run_index+1] = DirtyEnum::NeedCompute;
                }
            }
        }
        compute_count
    }
}

pub fn generate_linear_exec(count: usize) -> LinearExec {
    let nodes = (0..count-1).map(|_| {
        Box::new(AddOneAsDynCall{}) as Box<dyn DynCall>
    });

    let firstnode = vec![Box::new(ZeroAsDynCall{}) as Box<dyn DynCall>];

    let concat = firstnode.into_iter().chain(nodes);

    LinearExec::new(concat)
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

        let mut exec = LinearExec {
            store: DynStorage::new(2),
            dirty: DynDirty::new(2),
            nodes: vec![Box::new(one_compute), Box::new(addone_compute)]
        };

        let computed = exec.run();

        assert_eq!(computed, 2);
    
        let value1 = exec.value(1);
        if let Some(value1) = value1 {
            let value1_box = value1.downcast_ref::<i32>().unwrap();
            assert_eq!(*value1_box, 2);
        } else {
            assert!(false);
        }
        
    }

    #[test]
    fn test_dyn_chain() {
        const CHAIN_LENGTH: usize = 10;
        let mut exec = generate_linear_exec(CHAIN_LENGTH);
        let count = exec.run();
        assert_eq!(count,CHAIN_LENGTH);
        assert_eq!(exec.value(9).unwrap().downcast_ref::<i32>(),Some(&9));
        assert_eq!(exec.value(9).unwrap().downcast_ref::<i32>(),Some(&9));

        exec.set_runnable(0);
        let count = exec.run();
        assert_eq!(count,CHAIN_LENGTH);
    }
}