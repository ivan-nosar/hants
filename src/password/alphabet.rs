pub fn get_sorted_alphabet_chars(symbol_classes: String) -> Result<Vec<char>, String> {
    if symbol_classes.is_empty() {
        return Err("alphabet is empty".to_string());
    }
    
    let mut alphabet = String::new();
    for ch in symbol_classes.chars() {
        let symbols = match ch {
            'a' => "abcdefghijklmnopqrstuvwxyz",
            'A' => "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            'n' => "0123456789",
            'b' => "()<>[]{}",
            'q' => "'\"`",
            'p' => "!?.,;:",
            'm' => "+-*/=",
            'w' => " \t\n\r",
            's' => "\\^~@$&%_",
            other => return Err(format!("unknown symbol class: '{}'", other)),
        };
        alphabet.push_str(symbols);
    }

    let mut chars: Vec<char> = alphabet.chars().collect();
    chars.sort();
    
    Ok(chars)
}

#[cfg(test)]
mod tests {
    use super::get_sorted_alphabet_chars;

    /// Canonical symbol set for each supported class, mirroring the documented spec.
    fn class_symbols(class: char) -> &'static str {
        match class {
            'a' => "abcdefghijklmnopqrstuvwxyz",
            'A' => "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            'n' => "0123456789",
            'b' => "()<>[]{}",
            'q' => "'\"`",
            'p' => "!?.,;:",
            'm' => "+-*/=",
            'w' => " \t\n\r",
            's' => "\\^~@$&%_",
            other => panic!("test referenced unknown class '{other}'"),
        }
    }

    #[test]
    fn returns_err_for_empty_symbol_classes() {
        assert!(get_sorted_alphabet_chars(String::new()).is_err());
    }

    #[test]
    fn returns_sorted_chars_containing_all_class_symbols() {
        let cases = [
            "a", "A", "n", "b", "q", "p", "m", "w", "s", "aA", "aAn", "nbm", "aAnbqpms",
        ];

        for case in cases {
            let result = get_sorted_alphabet_chars(case.to_string())
                .unwrap_or_else(|e| panic!("expected Ok for '{case}', got Err: {e}"));

            // The returned characters must be in ascending order.
            let mut expected_sorted = result.clone();
            expected_sorted.sort();
            assert_eq!(result, expected_sorted, "result for '{case}' must be sorted");

            // Every symbol of every requested class must be present.
            for class in case.chars() {
                for symbol in class_symbols(class).chars() {
                    assert!(
                        result.contains(&symbol),
                        "result for '{case}' is missing {symbol:?} from class '{class}'"
                    );
                }
            }
        }
    }

    #[test]
    fn returns_equal_result_for_differently_ordered_classes() {
        let ascending = get_sorted_alphabet_chars("aAn".to_string()).unwrap();
        let shuffled = get_sorted_alphabet_chars("nAa".to_string()).unwrap();
        assert_eq!(ascending, shuffled);
    }

    #[test]
    fn returns_err_when_any_class_is_unknown() {
        let cases = ["Z", "aZ", "aAnX", "1", "nq1", "?"];

        for case in cases {
            assert!(
                get_sorted_alphabet_chars(case.to_string()).is_err(),
                "expected Err for '{case}'"
            );
        }
    }
}