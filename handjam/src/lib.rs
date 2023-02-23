#[derive(Default)]
pub struct MutInt {
    value: i32,
}

pub struct MutIntNoDefault {
    value: i32,
}

pub fn three() -> i32 {
    3
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