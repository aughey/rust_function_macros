fn main() {
    println!("Hello, world!");
}

// Create the graph:
// two
// | \
// |  \
// |   \-> add_double -> multiple_double 
// |  /                /
// | /                /
// three-------------/
fn exec()
{

}

struct GraphStore {
    
}

fn two() -> f64 {
    2.0
}

fn three() -> f64 {
    3.0
}

fn add_double(x: f64, y: f64) -> f64 {
    (x + y)
}

fn multiple_double(x: f64, y: f64) -> f64 {
    (x * y)
}

fn divide_double(x: f64, y: f64) -> f64 {
    (x / y)
}