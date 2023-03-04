#[macro_export]
macro_rules! run_operationzz {
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

#[macro_export]
macro_rules! run_operation {
    ( 
        runstate: $runstate:expr,
        output: $output:expr,
        function: $function:expr,
        children: $($children:expr),*
    ) => {
        if $runstate == DirtyEnum::NeedCompute {
            $output = {
                $runstate = DirtyEnum::Clean;
                Some($function())
            };
            $( $children = DirtyEnum::NeedCompute; )*
        }
    };
    ( 
        runstate: $runstate:expr,
        output: $output:expr,
        inputs: ($($vars:tt),*) = ($($states:expr),*),
        function: $function:path,
        children: $($children:expr),*
    ) => {
        if $runstate == DirtyEnum::NeedCompute {
            $output = if let ($(Some($vars)),*) = ($($states),*) {
                $runstate = DirtyEnum::Clean;
                Some($function($($vars),*))
            } else {
                $runstate = DirtyEnum::Stale;
                None
            };
            $( $children = DirtyEnum::NeedCompute; )*
        }
    };
}