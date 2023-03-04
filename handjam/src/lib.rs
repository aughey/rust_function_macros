pub mod macros;
pub mod gentest;
pub mod dyn_call;

#[derive(Default)]
pub struct MutInt {
    value: i32,
}

pub struct MutIntNoDefault {
    value: i32,
}

pub fn zero() -> i32 {
    0
}

pub fn one() -> i32 {
    1
}

pub fn two() -> i32 {
    2
}

pub fn two_optional() -> Option<i32> {
    Some(2)
}

#[derive(Copy,Clone)]
pub struct NullError;

pub fn one_result() -> Result<i32, NullError> {
    Ok(1)
}

pub fn three() -> i32 {
    3
}

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn mutable_int(value: i32) -> MutInt {
    MutInt { value }
}

pub fn mutable_int_no_default(value: i32) -> MutIntNoDefault {
    MutIntNoDefault { value }
}

pub fn tick(mut_int: &mut MutInt) -> i32 {
    mut_int.value += 1;
    mut_int.value
}

pub fn tick_no_default(mut_int: &mut MutIntNoDefault) -> i32 {
    mut_int.value += 1;
    mut_int.value
}

pub fn int_to_string(value: i32) -> String {
    value.to_string()
}

pub fn string_slice(string: &str) -> &str {
    &string[1..]
}

pub fn print(string: &str) {
    println!("{}", string);
}
