use crate::characters::*;
use crate::escape::Escape;

/// Count consecutive backslashes preceding a position
fn count_preceding_backslashes(letters: &[char], position: usize) -> usize {
    let mut count = 0;
    let mut check_index = position;

    while check_index > 0 && Escape::test(&letters[check_index - 1]) {
        count += 1;
        check_index -= 1;
    }

    count
}

// Representation of a single string
pub struct SingleString;

/// Implementation of single string test and tokenize methods.
impl SingleString {
    /// Return whether a character is a single string token `'`
    ///
    /// # Example
    ///
    /// ```
    /// use postcss_rs_tokenizer::string::SingleString;
    ///
    /// assert_eq!(SingleString::test(&'\''), true);
    /// ```
    pub fn test(letter: &char) -> bool {
        return matches!(letter, SINGLE_QUOTE);
    }

    /// Tokenize the string contained within single quotation marks like `'word'`.
    pub fn tokenize(letters: &[char]) -> usize {
        let count = letters.len();

        // Start from index 1 (after opening quote) and look for closing quote
        let mut index = 1;
        while index < count {
            if letters[index] == *SINGLE_QUOTE {
                // Found a quote - check if it's escaped by counting preceding backslashes
                let backslash_count = count_preceding_backslashes(letters, index);

                // Even number of backslashes (including 0) means quote is NOT escaped
                match backslash_count % 2 {
                    0 => return index, // Found unescaped closing quote
                    _ => index += 1,   // Quote is escaped, continue
                }
            } else {
                index += 1;
            }
        }

        // No closing quote found - return length minus 1 (the opening quote)
        return count - 1;
    }
}

pub struct DoubleString;

impl DoubleString {
    /// Return whether a character is a double string token `"`
    pub fn test(letter: &char) -> bool {
        return matches!(letter, DOUBLE_QUOTE);
    }

    /// Tokenize the string contained within double quotation marks like `"word"`.
    pub fn tokenize(letters: &[char]) -> usize {
        let count = letters.len();

        // Start from index 1 (after opening quote) and look for closing quote
        let mut index = 1;
        while index < count {
            if letters[index] == *DOUBLE_QUOTE {
                // Found a quote - check if it's escaped by counting preceding backslashes
                let backslash_count = count_preceding_backslashes(letters, index);

                // Even number of backslashes (including 0) means quote is NOT escaped
                match backslash_count % 2 {
                    0 => return index, // Found unescaped closing quote
                    _ => index += 1,   // Quote is escaped, continue
                }
            } else {
                index += 1;
            }
        }

        // No closing quote found - return length minus 1 (the opening quote)
        return count - 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_identifies_a_single_quote() {
        assert!(SingleString::test(&'\''));
    }

    #[test]
    fn it_does_not_identify_a_double_quote() {
        assert!(!SingleString::test(&'"'));
    }

    #[test]
    fn it_identifies_a_double_quote() {
        assert!(DoubleString::test(&'"'));
    }

    #[test]
    fn it_does_not_identify_a_single_quote() {
        assert!(!DoubleString::test(&'\''));
    }

    #[test]
    fn it_tokenizes_double_string() {
        let letters: Vec<char> = "\"'hi'\"".chars().collect();
        assert_eq!(DoubleString::tokenize(&letters[..]), 5);
    }

    #[test]
    fn it_tokenizes_single_string() {
        let letters: Vec<char> = "'\"hi\"'".chars().collect();
        assert_eq!(SingleString::tokenize(&letters[..]), 5);
    }
}
