pub fn format_with_commas(num: i32) -> String {
    let s = num.to_string();
    let len = s.len();
    if len > 3 {
        let mut result = String::new();
        let mut count = 0;

        // Traverse the string from the end and insert commas every 3 digits
        for c in s.chars().rev() {
            if count == 3 {
                result.push(',');
                count = 0;
            }
            result.push(c);
            count += 1;
        }

        result.chars().rev().collect() // Reverse to get the correct order
    } else {
        s
    }
}
