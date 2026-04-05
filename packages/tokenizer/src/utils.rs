/// Decrements `index` by one, saturating at zero.
///
/// This helper prevents underflow when stepping back one character during
/// tokenization.  It is used to convert an exclusive-end position into an
/// inclusive-end position.
///
/// # Examples
/// ```
/// use postcss_rs_tokenizer::utils::back;
/// assert_eq!(back(0), 0);
/// assert_eq!(back(5), 4);
/// ```
pub fn back(index: usize) -> usize {
    if index == 0 {
        return index;
    }

    index - 1
}

/// Unwraps an `Option<usize>` position, falling back to `len`.
///
/// When a search through a byte slice produces `None` (i.e. the pattern
/// was not found before the end of the slice), this function returns `len`
/// to indicate "end of input".
///
/// # Examples
/// ```
/// use postcss_rs_tokenizer::utils::unwrap_position;
/// assert_eq!(unwrap_position(Some(2), 5), 2);
/// assert_eq!(unwrap_position(None, 5), 5);
/// ```
pub fn unwrap_position(result: Option<usize>, len: usize) -> usize {
    match result {
        Some(value) => value,
        None => len,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_does_not_panic() {
        assert_eq!(back(0), 0)
    }

    #[test]
    fn test_back_positive() {
        assert_eq!(back(1), 0);
        assert_eq!(back(5), 4);
        assert_eq!(back(100), 99);
    }

    #[test]
    fn test_unwrap_position_some() {
        assert_eq!(unwrap_position(Some(0), 5), 0);
        assert_eq!(unwrap_position(Some(3), 5), 3);
    }

    #[test]
    fn test_unwrap_position_none_returns_length() {
        assert_eq!(unwrap_position(None, 5), 5);
    }

    #[test]
    fn test_unwrap_position_empty_slice() {
        assert_eq!(unwrap_position(None, 0), 0);
    }
}
