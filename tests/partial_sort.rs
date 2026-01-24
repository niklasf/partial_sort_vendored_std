use partial_sort_bench::partial_sort;

#[test]
fn test_partial_sort() {
    let mut v = [1, 2, 3, 1, 2, 3, 4, 0];
    partial_sort(&mut v, ..3, |a, b| a.lt(b));
    assert_eq!(v[..3], [0, 1, 1]);
}
