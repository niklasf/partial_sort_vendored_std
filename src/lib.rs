#![feature(cfg_select)]
#![feature(core_intrinsics)]
#![feature(slice_range)]

mod select;
mod smallsort;

use std::cfg_select;
use std::intrinsics;
use std::mem;
use std::ops::{Range, RangeBounds};
use std::slice;

use crate::select::partition_at_index;
use crate::smallsort::insertion_sort_shift_left;

/// Unstable sort called ipnsort by Lukas Bergdoll and Orson Peters.
/// Design document:
/// <https://github.com/Voultapher/sort-research-rs/blob/main/writeup/ipnsort_introduction/text.md>
///
/// Upholds all safety properties outlined here:
/// <https://github.com/Voultapher/sort-research-rs/blob/main/writeup/sort_safety/text.md>
#[inline(always)]
pub fn sort<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    // Arrays of zero-sized types are always all-equal, and thus sorted.
    if mem::size_of::<T>() == 0 {
        return;
    }

    // Instrumenting the standard library showed that 90+% of the calls to sort
    // by rustc are either of size 0 or 1.
    let len = v.len();
    if intrinsics::likely(len < 2) {
        return;
    }

    cfg_select! {
        any(feature = "optimize_for_size", target_pointer_width = "16") => {
            heapsort::heapsort(v, is_less);
        }
        _ => {
            // More advanced sorting methods than insertion sort are faster if called in
            // a hot loop for small inputs, but for general-purpose code the small
            // binary size of insertion sort is more important. The instruction cache in
            // modern processors is very valuable, and for a single sort call in general
            // purpose code any gains from an advanced method are cancelled by i-cache
            // misses during the sort, and thrashing the i-cache for surrounding code.
            const MAX_LEN_ALWAYS_INSERTION_SORT: usize = 20;
            if intrinsics::likely(len <= MAX_LEN_ALWAYS_INSERTION_SORT) {
                insertion_sort_shift_left(v, 1, is_less);
                return;
            }

            ipnsort(v, is_less);
        }
    }
}

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
    if std::mem::size_of::<T>() == 0 {
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
