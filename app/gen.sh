value=1
while [ $value -le 1000 ]; do
    prev=$((value - 1))
    next=$((value + 1))
    echo "run_operation!(runnable, $value, store.value[$value], operations::add_one(store.value[$prev]), run_count, $next);"
    value=$((value + 1))
    prev=$((value - 1))
    next=$((value + 1))
done

echo "run_operation!(runnable, $value, store.value[$value], operations::add_one(store.value[$prev]), run_count);"
