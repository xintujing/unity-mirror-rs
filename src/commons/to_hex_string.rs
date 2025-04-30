pub trait ToHexString {
    #[allow(unused)]
    fn to_hex_string(&self, seg: &str, uppercase: bool) -> String;
}

impl ToHexString for Vec<u8> {
    fn to_hex_string(&self, seg: &str, uppercase: bool) -> String {
        self.iter()
            .map(|byte| format!("{:02x}", byte))
            .map(|s| if uppercase { s.to_uppercase() } else { s })
            .collect::<Vec<String>>()
            .join(seg)
    }
}
