use std::collections::HashMap;

pub struct Component {
    factories: HashMap<String, HashMap<String, fn()>>,
}
