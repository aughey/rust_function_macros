

#[cfg(test)]
mod tests {
    use ive::dyn_call::box_dyn_call;

 
    #[test]
    fn test_execution() {
        let nodes = vec![box_dyn_call(crate::OneDynCall {})];
        let mut exec = ive::dyn_call::DynLinearExec::new_linear_chain(nodes.into_iter());

        let count = exec.run().unwrap();
        assert_eq!(count, 1);
        let value1 = exec.value::<i32>(0).unwrap();
        assert_eq!(*value1, 1);
    }

    #[test]
    fn test_introspection() {
        let one = crate::OneDynCall {};
        let info = &one as &dyn ive::dyn_call::DynInfo;

        let ot = info.output_type();
        assert_eq!(ot, &["i32"]);
        let inputs = info.inputs();
        assert_eq!(inputs.len(), 0);

        let add = crate::AddDynCall {};
        let info = &add as &dyn ive::dyn_call::DynInfo;

        let ot = info.output_type();
        assert_eq!(ot, &["i32"]);
        let inputs = info.inputs();
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].name, "a".to_string());
        assert_eq!(inputs[0].kind, &["i32"]);
        assert_eq!(inputs[1].name, "b".to_string());
        assert_eq!(inputs[1].kind, &["i32"]);
    }

}