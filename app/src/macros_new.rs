#[macro_export]
macro_rules! run_operation_new {
    ( $runnable_array:expr, $my_index:expr, $out:expr, $op:expr, $inputs:expr, $count:expr, $dependencies:expr ) => {
        if $runnable_array[$my_index] {
           $runnable_array[$my_index] = false;
            let result = stringify!($op)();
            $out = Some(result);
            $count += 1;
        //    $(
        //        $runnable_array[$dependencies] = true;
        //     )*
        }
    };
}