//! Helper function for combining Option-wrapped values

/// Given two Option-wrapped values:
/// - if both are `None`, returns `None`
/// - if one and only one has a value, returns that value in a `Some`
/// - if both have values, applies the combining operation `op` with the two contained values
#[inline]
pub fn match_opts<T, F>(a: Option<T>, b: Option<T>, op: F) -> Option<T>
where
    F: Fn(T, T) -> T,
{
    match (a, b) {
        (None, None) => None,
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        (Some(a), Some(b)) => Some(op(a, b)),
    }
}
