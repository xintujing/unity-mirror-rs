pub(crate) mod random_string;
pub(crate) mod string_case;

#[allow(unused)]
pub fn write_to_file(prefix: &str, value: String) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    std::fs::write(format!("tmp/{}_{}.rs", prefix, timestamp), value).expect("write file failed");
}
