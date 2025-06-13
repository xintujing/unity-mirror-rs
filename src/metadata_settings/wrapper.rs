use crate::commons::Object;
use serde::Deserialize;
use serde_json::{Error, Value};
use std::any::Any;

pub trait SettingsParser {
    fn parse(json: Value) -> Result<Box<Self>, Error>
    where
        Self: Sized + for<'a> Deserialize<'a>,
    {
        match Self::deserialize(json) {
            Ok(value) => Ok(Box::new(value)),
            Err(err) => Err(err),
        }
    }
}

impl<T: Settings + Send + Sync + 'static> SettingsParser for T {}

pub trait SettingsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Settings + Send + Sync + 'static> SettingsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait Settings:
    Any + Send + Sync + SettingsParser + SettingsAny + Object + SettingsClone
{
    // fn to<T>(&self) -> &'static T {
    //     self.as_any().downcast_ref::<T>().unwrap()
    // }
}

pub trait SettingsClone {
    fn clone_box(&self) -> Box<dyn Settings>;
}

impl<T: Settings + Clone + 'static> SettingsClone for T {
    fn clone_box(&self) -> Box<dyn Settings> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Settings> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// #[macro_export]
// macro_rules! settings_wrapper_register {
//     ($struct_ident:ident as $wrapper_path:path) => {
//         impl crate::metadata_settings::settings_wrapper::Settings for $struct_ident {}
//
//         paste::paste! {
//             #[ctor::ctor]
//             #[allow(non_snake_case, unused)]
//             fn [<$struct_ident _register>] (){
//                 $wrapper_path::register::<$struct_ident>();
//             }
//         }
//     };
// }
