pub mod macros;
pub mod gentest;

#[derive(Default)]
pub struct MutInt {
    value: i32,
}

pub struct MutIntNoDefault {
    value: i32,
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

pub fn one_result() -> Result<i32, ()> {
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

pub fn string_slice(string: &String) -> &str {
    &string[..]
}

pub fn print(string: &str) {
    println!("{}", string);
}
