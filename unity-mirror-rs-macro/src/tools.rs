pub(crate) fn generate_unique_string(len: usize) -> String {
    loop {
        let uuid = uuid::Uuid::new_v4().to_string();
        let mut chars = uuid.chars();
        if let Some(first_char) = chars.next() {
            if first_char.is_alphabetic() {
                return first_char.to_string() + chars.take(len - 1).collect::<String>().as_str();
            }
        }
    }
}
