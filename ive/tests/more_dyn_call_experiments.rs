use std::any::{Any};

fn add_two_numbers(a: u32, b: u32) -> u32 {
    a + b
}


// fn get_value<T: 'static>(value: &dyn Any) -> Option<T> {
//     if value.type_id() == TypeId::of::<T>() {
//         match value.downcast_ref::<T>() {
//             Some(val) => {
//                 if std::any::type_name::<T>().ends_with("&") {
//                     // Return reference
//                     Some(val)
//                 } else {
//                     // Return copy
//                     Some(*val)
//                 }
//             }
//             None => None,
//         }
//     } else {
//         None
//     }
// }


fn add_two_numbers_dyn(a: &dyn Any, b: &dyn Any) -> Box<dyn Any> {
    let a = a.downcast_ref::<u32>().unwrap();
    let b = b.downcast_ref::<u32>().unwrap();
    let r = add_two_numbers(*a, *b);
    Box::new(r)
}

// fn add_two_numbers_infer(a: &dyn Any, b: &dyn Any) -> Box<dyn Any> {
//     let r = add_two_numbers(get_value(a).unwrap(), get_value(b).unwrap());
//     Box::new(r)
// }


#[test]
fn test_something() {
    assert_eq!(1,1);
}

#[test]
fn test_add_dyn() {
    let a : Box<dyn Any> = Box::new(1u32);
    let b : Box<dyn Any> = Box::new(2u32);
    let r = add_two_numbers_dyn(a.as_ref(), b.as_ref());
    let r = r.downcast_ref::<u32>().unwrap();
    assert_eq!(r, &3);

}