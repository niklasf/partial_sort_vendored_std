use std::ops::RangeBounds;

/// Unstable partial sort the range `start..end`, after which it's guaranteed that:
///
/// 1. Every element in `v[..start]` is smaller than or equal to
/// 2. Every element in `v[start..end]`, which is sorted, and smaller than or equal to
/// 3. Every element in `v[end..]`.
#[inline]
pub fn partial_sort<T, F, R>(v: &mut [T], range: R, mut is_less: F)
where
    F: FnMut(&T, &T) -> bool,
    R: RangeBounds<usize>,
{
    // Arrays of zero-sized types are always all-equal, and thus sorted.
    if T::IS_ZST {
        return;
    }

    let len = v.len();
    let Range { start, end } = slice::range(range, ..len);

    if end - start <= 1 {
        // Empty range or single element. This case can be resolved in at most
        // single partition_at_index call, without further sorting.

        if end == 0 || start == len {
            // Do nothing if it is an empty range at start or end: all guarantees
            // are already upheld.
            return;
        }

        partition_at_index(v, start, &mut is_less);
        return;
    }

    // A heuristic factor to decide whether to partition the slice or not.
    // If the range bound is close to the edges of the slice, it's not worth
    // partitioning first.
    const PARTITION_THRESHOLD: usize = 8;
    let mut v = v;
    if end + PARTITION_THRESHOLD <= len {
        v = partition_at_index(v, end - 1, &mut is_less).0;
    }
    if start >= PARTITION_THRESHOLD {
        v = partition_at_index(v, start, &mut is_less).2;
    }

    sort(v, &mut is_less);
}
