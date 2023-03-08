//use ive_macros::make_dynamicable;

use ive_macros::make_dynamicable;

pub mod macros;
pub mod gentest;
pub mod graph;
pub mod descriptive_ive;
pub mod linear_execution;

#[make_dynamicable()]
pub fn zero() -> i32 {
    0
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
pub fn two_optional() -> Option<i32> {
    Some(2)
}

#[derive(Copy,Clone)]
pub struct NullError;

pub fn one_result() -> Result<i32, NullError> {
    Ok(1)
}

#[make_dynamicable()]
pub fn three() -> i32 {
    3
}

#[make_dynamicable()]
pub fn add_one(a: i32) -> i32 {
    a + 1
}

#[make_dynamicable()]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[make_dynamicable()]
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}




#[make_dynamicable()]
pub fn int_to_string(value: i32) -> String {
    value.to_string()
}


pub fn print(string: &str) {
    println!("{}", string);
}
