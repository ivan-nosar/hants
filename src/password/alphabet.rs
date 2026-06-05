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