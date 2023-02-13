#[macro_export]
macro_rules! run_operation {
( $runnable_array:expr, $my_index:expr, $out:expr, $op:expr, $count:expr ) => {
   if $runnable_array[$my_index] {
       $runnable_array[$my_index] = false;
        $out = Some($op);
        $count += 1;
   }
};
( $runnable_array:expr, $my_index:expr, $out:expr, $op:expr, $count:expr, $($dependencies:expr),* ) => {
    if $runnable_array[$my_index] {
       $runnable_array[$my_index] = false;
        $out = Some($op);
        $count += 1;
       $(
           $runnable_array[$dependencies] = true;
        )*
    }
};
}

