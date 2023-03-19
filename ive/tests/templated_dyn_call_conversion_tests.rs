use std::{any::Any, collections::HashMap};

use ive::dyn_call::{BoxedAny, InputGetter, OutputSetter};

fn extract_option_from<'a, T>(any: &'a dyn Any) -> anyhow::Result<Option<&'a T>>
where
    Option<T>: 'static,
{
    let invalue = any
        .downcast_ref::<Option<T>>()
        .ok_or(anyhow::anyhow!("Could not downcast"))?;
    let invalue = invalue.as_ref();

    Ok(invalue)
}

fn test_function(a: &u32, b: &u32) -> u32 {
    a + b
}

fn test_special(a: &u32, b: &u32) -> u32 {
    a * b
}

impl IntoDynCaller<DynCallMarker> for fn(&u32) -> u32 {
    fn into_dyn_caller(self) -> Box<DynamicCall> {
        Box::new(move |inputs, outputs| {
            outputs.some(0, 12345);
        })
    }
}

fn run_fn_2<A, B, R>(f: fn(&A, &B) -> R, a: &dyn Any, b: &dyn Any) -> Box<dyn Any>
where
    R: Any,
    A: 'static,
    B: 'static,
{
    let a = a.downcast_ref::<A>().unwrap();
    let b = b.downcast_ref::<B>().unwrap();
    let r = f(a, b);
    Box::new(r)
}

fn test_function_one_param(a: &u32) -> u32 {
    a * 2
}

type DynamicCall = dyn Fn(&InputGetter, &mut OutputSetter);

pub trait IntoDynCaller<Marker> {
    fn into_dyn_caller(self) -> Box<DynamicCall>;
}

impl<Func, A1, R> IntoDynCaller<(DynCallMarker, A1)> for Func
where
    Func: Fn(&A1) -> R + 'static,
    A1: 'static,
    R: 'static,
{
    fn into_dyn_caller(self) -> Box<DynamicCall> {
        Box::new(move |inputs, outputs| {
            let a = inputs.fetch::<A1>(0).unwrap();

            let out = (self)(a);
            outputs.some(0, out);
        })
    }
}


pub struct DynCallMarker;

impl<Func, A1, A2, R> IntoDynCaller<(DynCallMarker, A1, A2, R)> for Func
where
    Func: Fn(&A1, &A2) -> R+ 'static,
    A1: 'static,
    A2: 'static,
    R: 'static,
    //  A1: 'static,
    //   A2: 'static,
{
    fn into_dyn_caller(self) -> Box<DynamicCall> {
        Box::new(move |inputs, outputs| {
            let a = inputs.fetch::<A1>(0).unwrap();
            let b = inputs.fetch::<A2>(1).unwrap();
            let out = (self)(a, b);
            outputs.some(0, out);
        })
    }
}

#[derive(Default)]
struct Callers {
    callers: HashMap<String, Box<DynamicCall>>,
}
impl Callers {
    pub fn register<S, M>(&mut self, name: S, handler: impl IntoDynCaller<M>)
    where
        S: Into<String>,
    {
        let f = handler.into_dyn_caller();
        self.callers.insert(name.into(), f);
    }
}

// pub trait Handler<Args>: Clone + 'static {
//     type Args;
//     fn call(&self, args: &[Box<dyn Any>]) -> Box<dyn Any>;
// }

// impl<Func, RET, A> Handler<A> for Func
// where
//     Func: Fn(&A) -> RET + Clone + 'static,
//     RET: Any + 'static,
// {
//     type Args = A;
//     fn call(&self, args: &[Box<dyn Any>]) -> Box<dyn Any> {
//         let a = args[0].downcast_ref::<A>().unwrap();
//         let output = (self)(a);
//         Box::new(output)
//     }
// }

// impl<Func, RET, A, B> Handler<(A, B)> for Func
// where
//     Func: Fn(&A, &B) -> RET + Clone + 'static,
//     RET: Any + 'static,
// {
//     type Args = (A,B);
//     fn call(&self, args: &[Box<dyn Any>]) -> Box<dyn Any> {
//         let a = args[0].downcast_ref::<A>().unwrap();
//         let b = args[1].downcast_ref::<B>().unwrap();
//         let output = (self)(a, b);
//         Box::new(output)
//     }
// }

// type StoredHandler = Box<dyn Fn(&[Box<dyn Any>]) -> Box<dyn Any>>;
// struct Register {
//     handlers: HashMap<String, StoredHandler>,
// }
// impl Register {
//     pub fn reg<F, Args>(&mut self, name: &str, handler: F)
//     where
//         F: Handler<Args>,
//     {
//         let handler = handler.clone();
//         let handler = move |args: &[Box<dyn Any>]| handler.call(args);
//         self.handlers.insert(name.to_string(), Box::new(handler));
//     }

//     pub fn call(&self, name: &str, args: &[Box<dyn Any>]) -> Box<dyn Any> {
//         let handler = self.handlers.get(name).unwrap();
//         handler(args)
//     }
// }

// pub trait FromAny {
//     fn from_any<'a>(any: impl Any) -> &'a Self;
// }
// impl<T> FromAny for &T where T : Any  {
//     fn from_any(any: &Box<dyn Any>) -> &T {
//         any.downcast_ref::<T>().unwrap()
//     }
// }

#[test]
fn any_conversion() {
    let original = "John Aughey".to_string();

    let original = Some(original);

    let any: Box<dyn Any> = Box::new(original);
    assert!(any.is::<Option<String>>());

    let extracted = extract_option_from::<String>(any.as_ref()).unwrap();

    //assert!(extracted.is_ok());
    assert_eq!(extracted.unwrap(), "John Aughey");
}

#[test]
fn test_run_fn_2() {
    let a = Box::new(1u32) as Box<dyn Any>;
    let b = Box::new(2u32) as Box<dyn Any>;
    let r = run_fn_2(test_function, a.as_ref(), b.as_ref());
    assert_eq!(r.downcast_ref::<u32>().unwrap(), &3u32);
}

#[test]
fn test_from_any() {
    let a = Box::new(1u32) as Box<dyn Any>;

    let b: &u32 = a.downcast_ref::<u32>().unwrap();
    assert_eq!(b, &1u32);
}

#[test]
fn test_regisetr() {
    // let mut reg = Register {
    //     handlers: HashMap::new(),
    // };

    // reg.reg("test", |a: &u32| a + 1);
}

#[test]
fn test_test_function_one_param() {
    let res = test_function_one_param(&3);
    assert_eq!(res, 6);
}

#[test]
fn test_callers() {
    let mut callers = Callers::default();
    callers.register("test", test_function);
    callers.register("test_one", test_function_one_param);
    callers.register("test_special", test_special);

    let inputs = vec![Some(BoxedAny::new(5u32)), Some(BoxedAny::new(2u32))];
    let mut outputs: Vec<Option<BoxedAny>> = vec![None];

    assert!(outputs[0].is_none());
    {
        let input_getter = InputGetter::new(
            inputs.as_slice(),
            &[0, 1]);

        let mut output_setter = OutputSetter::new(
            outputs.as_mut_slice());

        let caller = callers.callers.get("test").unwrap();
        caller(&input_getter, &mut output_setter);
    }

    assert!(outputs[0].is_some());
    assert_eq!(outputs[0].as_ref().unwrap().value::<u32>().unwrap(), &7u32);

    {
        let input_getter = InputGetter::new(
            inputs.as_slice(),
            &[0]);

        let mut output_setter = OutputSetter::new(
            outputs.as_mut_slice());

        let caller = callers.callers.get("test_one").unwrap();
        caller(&input_getter, &mut output_setter);
    }

    assert!(outputs[0].is_some());
    assert_eq!(outputs[0].as_ref().unwrap().value::<u32>().unwrap(), &10u32);

    {
        let input_getter = InputGetter::new(
            inputs.as_slice(),
            &[0,1]);

        let mut output_setter = OutputSetter::new(
            outputs.as_mut_slice());

        let caller = callers.callers.get("test_special").unwrap();
        caller(&input_getter, &mut output_setter);
    }

    assert!(outputs[0].is_some());
    assert_eq!(outputs[0].as_ref().unwrap().value::<u32>().unwrap(), &12345u32);

}
