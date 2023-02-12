pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

pub fn three() -> u32 {
    3
}

pub fn four() -> u32 {
    4
}

pub fn copy_u32(input: u32) -> u32 {
    input
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
