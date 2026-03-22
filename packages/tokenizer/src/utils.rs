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

    return index - 1;
}

/// Unwraps an `Option<usize>` position, falling back to `letters.len()`.
///
/// When a search through a character slice produces `None` (i.e. the pattern
/// was not found before the end of the slice), this function returns the length
/// of the slice to indicate "end of input".
///
/// # Examples
/// ```
/// use postcss_rs_tokenizer::utils::unwrap_position;
/// let chars: Vec<char> = "hello".chars().collect();
/// assert_eq!(unwrap_position(Some(2), &chars), 2);
/// assert_eq!(unwrap_position(None, &chars), 5);
/// ```
pub fn unwrap_position(result: Option<usize>, letters: &[char]) -> usize {
    let position = match result {
        Some(value) => value,
        None => letters.len(),
    };

    return position;
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
        let chars: Vec<char> = "hello".chars().collect();
        assert_eq!(unwrap_position(Some(0), &chars), 0);
        assert_eq!(unwrap_position(Some(3), &chars), 3);
    }

    #[test]
    fn test_unwrap_position_none_returns_length() {
        let chars: Vec<char> = "hello".chars().collect();
        assert_eq!(unwrap_position(None, &chars), 5);
    }

    #[test]
    fn test_unwrap_position_empty_slice() {
        let chars: Vec<char> = Vec::new();
        assert_eq!(unwrap_position(None, &chars), 0);
    }
}
