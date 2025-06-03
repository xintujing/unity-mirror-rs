pub trait StringCase {
    fn to_snake_case(&self) -> String;

    fn to_camel_case(&self) -> String;
}

impl StringCase for str {
    fn to_snake_case(&self) -> String {
        to_snake_case(self)
    }

    fn to_camel_case(&self) -> String {
        to_camel_case(self)
    }
}

impl StringCase for String {
    fn to_snake_case(&self) -> String {
        to_snake_case(self)
    }

    fn to_camel_case(&self) -> String {
        to_camel_case(self)
    }
}

fn to_snake_case(val: &str) -> String {
    let mut result = String::new();
    for (i, c) in val.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}

fn to_camel_case(val: &str) -> String {
    val.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first_char) => first_char.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
