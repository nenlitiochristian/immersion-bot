pub fn format_with_commas(num: i32) -> String {
    // if negative, remove the negative mark and process it as a positive number
    let (mut str, is_negative) = match num < 0 {
        true => (num.abs().to_string(), true),
        false => (num.to_string(), false),
    };
    let len = str.len();

    if len > 3 {
        if num < 0 {}
        let mut result = String::new();
        let mut count = 0;

        for c in str.chars().rev() {
            if count == 3 {
                result.push(',');
                count = 0;  
            }
            result.push(c);
            count += 1;
        }

        if is_negative {
            result.push('-');
        }

        result.chars().rev().collect()
    } else {
        if is_negative {
            str = format!("-{str}");
        }
        str
    }
}
