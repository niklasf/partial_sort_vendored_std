/// Reorders the slice such that the element at `index` is at its final sorted position.
pub(crate) fn partition_at_index<T, F>(
    v: &mut [T],
    index: usize,
    mut is_less: F,
) -> (&mut [T], &mut T, &mut [T])
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();

    // Puts a lower limit of 1 on `len`.
    if index >= len {
        panic!(
            "partition_at_index index {} greater than length of slice {}",
            index, len
        );
    }

    if std::mem::size_of::<T>() == 0 {
        // Sorting has no meaningful behavior on zero-sized types. Do nothing.
    } else if index == len - 1 {
        // Find max element and place it in the last position of the array. We're free to use
        // `unwrap()` here because we checked that `v` is not empty.
        let max_idx = max_index(v, &mut is_less).unwrap();
        v.swap(max_idx, index);
    } else if index == 0 {
        // Find min element and place it in the first position of the array. We're free to use
        // `unwrap()` here because we checked that `v` is not empty.
        let min_idx = min_index(v, &mut is_less).unwrap();
        v.swap(min_idx, index);
    } else {
        cfg_select! {
            feature = "optimize_for_size" => {
                median_of_medians(v, &mut is_less, index);
            }
            _ => {
                partition_at_index_loop(v, index, None, &mut is_less);
            }
        }
    }

    let (left, right) = v.split_at_mut(index);
    let (pivot, right) = right.split_at_mut(1);
    let pivot = &mut pivot[0];
    (left, pivot, right)
}

/// Helper function that returns the index of the minimum element in the slice using the given
/// comparator function
fn min_index<T, F: FnMut(&T, &T) -> bool>(slice: &[T], is_less: &mut F) -> Option<usize> {
    slice
        .iter()
        .enumerate()
        .reduce(|acc, t| if is_less(t.1, acc.1) { t } else { acc })
        .map(|(i, _)| i)
}

/// Helper function that returns the index of the maximum element in the slice using the given
/// comparator function
fn max_index<T, F: FnMut(&T, &T) -> bool>(slice: &[T], is_less: &mut F) -> Option<usize> {
    slice
        .iter()
        .enumerate()
        .reduce(|acc, t| if is_less(acc.1, t.1) { t } else { acc })
        .map(|(i, _)| i)
}
