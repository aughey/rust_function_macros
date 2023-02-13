pub fn two() -> f64 {
    2.0
}

pub fn three() -> f64 {
    3.0
}

pub fn add_double(x: f64, y: f64) -> f64 {
    x + y
}

pub fn multiple_double(x: f64, y: f64) -> f64 {
    x * y
}

pub fn divide_double(x: f64, y: f64) -> f64 {
    x / y
}

pub fn add_one(x: u32) -> u32 {
    x + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two() {
        assert_eq!(two(), 2.0);
    }

    #[test]
    fn test_three() {
        assert_eq!(three(), 3.0);
    }

    #[test]
    fn test_add_double() {
        assert_eq!(add_double(1.0, 2.0), 3.0);
    }

    #[test]
    fn test_multiple_double() {
        assert_eq!(multiple_double(1.0, 2.0), 2.0);
    }

    #[test]
    fn test_divide_double() {
        assert_eq!(divide_double(1.0, 2.0), 0.5);
    }

    #[test]
    fn test_add_one() {
        assert_eq!(add_one(1), 2);
    }

 
}