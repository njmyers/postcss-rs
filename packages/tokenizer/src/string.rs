use crate::escape::Escape;

/// Count consecutive backslashes preceding a position
fn count_preceding_backslashes(bytes: &[u8], position: usize) -> usize {
    let mut count = 0;
    let mut check_index = position;

    while check_index > 0 && Escape::test(bytes[check_index - 1]) {
        count += 1;
        check_index -= 1;
    }

    count
}

// Representation of a single string
pub struct SingleString;

/// Implementation of single string test and tokenize methods.
impl SingleString {
    /// Return whether a byte is a single-quote `'`
    ///
    /// # Example
    ///
    /// ```
    /// use postcss_rs_tokenizer::string::SingleString;
    ///
    /// assert_eq!(SingleString::test(b'\''), true);
    /// ```
    pub fn test(byte: u8) -> bool {
        byte == b'\''
    }

    /// Tokenize the string contained within single quotation marks like `'word'`.
    pub fn tokenize(bytes: &[u8]) -> usize {
        let count = bytes.len();
        let mut index = 1;

        while index < count {
            if bytes[index] != b'\'' {
                index += 1;
                continue;
            }

            let backslash_count = count_preceding_backslashes(bytes, index);

            match backslash_count % 2 {
                0 => return index,
                _ => index += 1,
            }
        }

        return count - 1;
    }
}

pub struct DoubleString;

impl DoubleString {
    /// Return whether a byte is a double-quote `"`
    pub fn test(byte: u8) -> bool {
        byte == b'"'
    }

    /// Tokenize the string contained within double quotation marks like `"word"`.
    pub fn tokenize(bytes: &[u8]) -> usize {
        let count = bytes.len();
        let mut index = 1;

        while index < count {
            if bytes[index] != b'"' {
                index += 1;
                continue;
            }

            let backslash_count = count_preceding_backslashes(bytes, index);

            match backslash_count % 2 {
                0 => return index,
                _ => index += 1,
            }
        }

        return count - 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_identifies_a_single_quote() {
        assert!(SingleString::test(b'\''));
    }

    #[test]
    fn it_does_not_identify_a_double_quote() {
        assert!(!SingleString::test(b'"'));
    }

    #[test]
    fn it_identifies_a_double_quote() {
        assert!(DoubleString::test(b'"'));
    }

    #[test]
    fn it_does_not_identify_a_single_quote() {
        assert!(!DoubleString::test(b'\''));
    }

    #[test]
    fn it_tokenizes_double_string() {
        assert_eq!(DoubleString::tokenize(b"\"'hi'\""), 5);
    }

    #[test]
    fn it_tokenizes_single_string() {
        assert_eq!(SingleString::tokenize(b"'\"hi\"'"), 5);
    }
}
