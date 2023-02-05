#[derive(Default)]
pub struct Store {
    initial_string: String,
}

#[test]
fn test_exec() {
   fn run(store : &mut Store) {
    store.initial_string = "hello world".to_string();
    // 3rd through 7th characters
    let slc = &store.initial_string[3..7];

    assert_eq!(slc, "lo w");
   }

    let mut store = Store::default();

    assert_eq!(store.initial_string, "");

    run(&mut store);

    assert_eq!(store.initial_string, "hello world");
}