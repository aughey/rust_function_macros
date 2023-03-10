use std::{any::{Any}};

fn extract_option_from<'a, T>(any: &'a dyn Any) -> anyhow::Result<Option<&'a T>>
where
    Option<T>: 'static
{
    let invalue = any.downcast_ref::<Option<T>>().ok_or(anyhow::anyhow!("Could not downcast"))?;
    let invalue = invalue.as_ref();
    
    Ok(invalue)
}

fn test_function(a : &u32, b : &u32) -> u32 {
    a + b
}

fn run_fn_2<A,B,R>(f : fn(&A,&B) -> R, a : & dyn Any, b : &dyn Any) -> Box<dyn Any> where R : Any, A : 'static, B : 'static {
    let a = a.downcast_ref::<A>().unwrap();
    let b = b.downcast_ref::<B>().unwrap();
    let r = f(a,b);
    Box::new(r)
}


#[test]
fn any_conversion() {
    let original = "John Aughey".to_string();

    let original = Some(original);

    let any : Box<dyn Any> = Box::new(original);
    assert!(any.is::<Option<String>>());

    let extracted = extract_option_from::<String>(any.as_ref()).unwrap();

    //assert!(extracted.is_ok());
    assert_eq!(extracted.unwrap(), "John Aughey");
}

#[test]
fn test_run_fn_2() {
    let a = Box::new(1u32) as Box<dyn Any>;
    let b = Box::new(2u32) as Box<dyn Any>;
    let r = run_fn_2(test_function, &a, &b);
    assert_eq!(r.downcast_ref::<u32>().unwrap(), &3u32);
}