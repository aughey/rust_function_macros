use std::{any::Any, collections::HashMap};

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

pub trait FromAny {
    fn from_any<'a>(any: impl Any) -> &'a Self;
}
impl<T> FromAny for &T where T : Any  {
    fn from_any(any: &Box<dyn Any>) -> &T {
        any.downcast_ref::<T>().unwrap()
    }
}

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
