pub trait Namespace {
    fn get_namespace() -> &'static str;
    fn get_package() -> &'static str;
}

// impl<T> Namespace for nalgebra::Vector3<T> {
//     fn get_namespace() -> &'static str {
//         "UnityEngine"
//     }
//
//     fn get_prefix() -> &'static str {
//         ""
//     }
// }

#[cfg(test)]
mod tests {
    #[test]
    fn test1() {
        // let a = Vector3::<f64>::get_namespace();
    }
}
