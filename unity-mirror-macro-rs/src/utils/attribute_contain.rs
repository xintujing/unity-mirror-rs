use proc_macro::Ident;
use quote::ToTokens;
use syn::Attribute;
use syn::parse::Parse;

pub(crate) trait VecAttributeExpand {
    fn extract<T>(&self, attr_str: &str) -> Option<T>
    where
        T: Parse;

    fn contain(&self, attr_str: &str) -> bool;
}

impl VecAttributeExpand for Vec<Attribute> {
    fn extract<T>(&self, attr_str: &str) -> Option<T>
    where
        T: Parse,
    {
        for attr in self.iter() {
            if attr.path().get_ident().map(|ident| ident.to_string()) == Some(attr_str.to_string())
            {
                return attr.parse_args::<T>().ok();
            }
        }
        None
    }

    fn contain(&self, attr_str: &str) -> bool {
        for attr in self.iter() {
            if attr.path().get_ident().map(|ident| ident.to_string()) == Some(attr_str.to_string())
            {
                return true;
            }
        }
        false
    }
}
