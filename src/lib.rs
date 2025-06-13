#![allow(dead_code, unused)]
pub mod commons;
pub mod metadata_settings;
pub mod mirror;
pub mod transports;
pub mod unity_engine;

macro_rules! expand_macro {
    ($($name: ident),*) => {
        $(
            pub use unity_mirror_macro_rs::$name;
        )*
    };
}
// pub use unity_mirror_macro_rs;
expand_macro! {
    SyncState,
    network_behaviour,
    namespace,
    MetadataSettingsWrapper,
    settings_wrapper_register,
    ancestor_on_serialize,
    ancestor_on_deserialize,
    parent_on_serialize,
    parent_on_deserialize,
    NetworkMessage,
    CallbackProcessor,
    network_manager,
    NetworkManagerFactory,
    authenticator_factory,
    command,
    client_rpc,
    target_rpc,
    extends,
    action
}