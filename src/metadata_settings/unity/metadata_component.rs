static METADATA_COMPONENT_REGISTERS: once_cell::sync::Lazy<
    std::sync::Mutex<
        std::collections::HashMap<
            &'static str,
            fn(
                serde_json::Value,
            )
                -> Result<Box<dyn crate::metadata_settings::wrapper::Settings>, serde_json::Error>,
        >,
    >,
> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

pub struct MetadataComponentWrapper {
    value: std::collections::HashMap<
        std::any::TypeId,
        Vec<Box<dyn crate::metadata_settings::wrapper::Settings>>,
    >,
}
impl MetadataComponentWrapper {
    pub fn register<
        T: crate::metadata_settings::wrapper::Settings + 'static + for<'a> serde::Deserialize<'a>,
    >() {
        let name = T::get_namespace();
        let type_name = std::any::type_name::<T>();
        println!("Register component: {} {}", type_name, name);
        let parser = |value: serde_json::Value| -> Result<
            Box<dyn crate::metadata_settings::wrapper::Settings>,
            serde_json::Error,
        > {
            T::parse(value).map(|c| c as Box<dyn crate::metadata_settings::wrapper::Settings>)
        };
        if let Ok(mut component_registry) = METADATA_COMPONENT_REGISTERS.lock() {
            if component_registry.contains_key(name) {
                panic!("Component already registered: {}", name);
            }
            component_registry.insert(name, parser);
        }
    }
    pub fn list<T: crate::metadata_settings::wrapper::Settings>(&self) -> Vec<&T> {
        if let Some(components) = self.value.get(&std::any::TypeId::of::<T>()) {
            return components
                .iter()
                .map(|c| c.as_any().downcast_ref::<T>().unwrap())
                .collect::<Vec<_>>();
        }
        panic!("Component not found: {}", std::any::type_name::<T>());
    }
}

impl<'de> serde::Deserialize<'de> for MetadataComponentWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut wrapper = Self {
            value: Default::default(),
        };

        let component =
            std::collections::HashMap::<String, Vec<serde_json::Value>>::deserialize(deserializer)?;

        for (key, values) in component.iter() {
            if let Ok(component_registry) = METADATA_COMPONENT_REGISTERS.lock() {
                if let Some(parser) = component_registry.get(key.as_str()) {
                    for value in values.iter() {
                        let component = parser(value.clone()).map_err(|err| {
                            serde::de::Error::custom(format!(
                                "Component {} parse error: {}",
                                key, err
                            ))
                        })?;

                        let id = component.as_any().type_id();
                        if !wrapper.value.contains_key(&id) {
                            wrapper.value.insert(id, vec![]);
                        }
                        if let Some(values) = wrapper.value.get_mut(&id) {
                            values.push(component);
                        }
                    }
                } else {
                    panic!("Component not found: {}", key);
                }
            }
        }

        Ok(wrapper)
    }
}
