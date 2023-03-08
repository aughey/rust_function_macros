trait NodeInfo {
    fn kind(&self) -> &str;
    fn inputs(&self);
    fn outputs(&self);
}

trait Port {
    fn name(&self) -> &str;
    fn kind<'a, CB, IT>(&self, tokens_fn: CB) -> IT
    where
        CB: Fn(IT) -> (),
        IT: Iterator<Item = &'a str>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MyObject;
    impl<'a> NodeInfo<'a> for MyObject {
        type PortIterator = impl Iterator<Item = &'a Port<'a>>;
        fn kind(&self) -> &str {
            "MyObject"
        }
        fn inputs(&self) -> Self::PortIterator {
            [&Port {
                name: "a",
                kind: &["int", "foo"],
            }]
            .into_iter()
        }
        fn outputs(&self) -> Self::PortIterator {
            [&Port {
                name: "value",
                kind: &["int"],
            }]
            .into_iter()
        }
    }

    #[test]
    fn it_works() {
        let obj = MyObject;
        let inputs = obj.inputs().collect::<Vec<_>>();
        let outputs = obj.outputs().collect::<Vec<_>>();
        assert_eq!(inputs.len(), 1);
        assert_eq!(outputs.len(), 1);
        assert_eq!(inputs[0].name, "a");
        assert_eq!(inputs[0].kind, &["int", "foo"]);
        assert_eq!(outputs[0].name, "value");
        assert_eq!(outputs[0].kind, &["int"]);
    }
}
