pub fn back(index: usize) -> usize {
    if index == 0 {
        return index;
    }

    return index - 1;
}

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
}
