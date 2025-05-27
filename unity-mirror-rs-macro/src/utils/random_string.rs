pub(crate) fn random_string(length: usize) -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..length)
        .map(|_| rng.sample(rand::distr::Alphanumeric) as char)
        .collect()
}
