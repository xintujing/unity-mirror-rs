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

expand_macro! {
    MetadataSettingsWrapper,
    settings_wrapper_register,
    network_manager,
    extends,
    action
}

// use unity_mirror_rs::macro_namespace::*;
// use crate::macro_namespace::*;
pub mod macro_namespace {
    pub use crate::commons::Object;
    pub use unity_mirror_macro_rs::namespace;
}


// use unity_mirror_rs::macro_network_message::*;
// use crate::macro_network_message::*;
pub mod macro_network_message {
    pub use super::mirror::message::NetworkMessage;
    pub use unity_mirror_macro_rs::NetworkMessage;
}


// use unity_mirror_rs::macro_callback_processor::*;
// use crate::macro_callback_processor::*;
pub mod macro_callback_processor {
    pub use super::mirror::CallbackProcessor;
    pub use super::mirror::TransportChannel;
    pub use super::mirror::TransportError;
    pub use unity_mirror_macro_rs::CallbackProcessor;
}


// use unity_mirror_rs::macro_network_manager::*;
// use crate::macro_network_manager::*;
pub mod macro_network_manager {
    pub use unity_mirror_macro_rs::network_manager;
    pub use unity_mirror_macro_rs::NetworkManagerFactory;
}

// use unity_mirror_rs::macro_network_behaviour::*;
// use crate::macro_network_behaviour::*;
pub mod macro_network_behaviour {
    pub use super::commons::RevelArc;
    pub use super::commons::RevelWeak;
    pub use super::mirror::DataTypeDeserializer;
    pub use super::mirror::DataTypeSerializer;
    pub use super::mirror::NetworkBehaviour;
    pub use super::mirror::NetworkBehaviourBase;
    pub use super::mirror::NetworkBehaviourDeserializer;
    pub use super::mirror::NetworkBehaviourFactory;
    pub use super::mirror::NetworkBehaviourOnDeserializer;
    pub use super::mirror::NetworkBehaviourOnSerializer;
    pub use super::mirror::NetworkBehaviourSerializer;
    pub use super::mirror::NetworkConnectionToClient;
    pub use super::mirror::NetworkIdentity;
    pub use super::mirror::NetworkReader;
    pub use super::mirror::NetworkWriter;
    pub use super::mirror::NetworkWriterPool;
    pub use super::mirror::RemoteProcedureCalls;
    pub use super::mirror::StableHash;
    pub use super::mirror::SyncDirection;
    pub use super::mirror::SyncMode;
    pub use super::mirror::SyncObject;
    pub use super::mirror::TBaseNetworkBehaviour;
    pub use super::mirror::TransportChannel;
    pub use unity_mirror_macro_rs::ancestor_on_deserialize;
    pub use unity_mirror_macro_rs::ancestor_on_serialize;
    pub use unity_mirror_macro_rs::client_rpc;
    pub use unity_mirror_macro_rs::command;
    pub use unity_mirror_macro_rs::network_behaviour;
    pub use unity_mirror_macro_rs::parent_on_deserialize;
    pub use unity_mirror_macro_rs::parent_on_serialize;
    pub use unity_mirror_macro_rs::target_rpc;
    pub use unity_mirror_macro_rs::SyncState;
}

// use unity_mirror_rs::macro_authenticator_factory::*;
// use crate::macro_authenticator_factory::*;
pub mod macro_authenticator_factory {
    pub use super::commons::RevelArc;
    pub use super::commons::RevelWeak;
    pub use super::mirror::AuthenticatorBase;
    pub use super::mirror::AuthenticatorFactory;
    pub use super::mirror::NetworkConnectionToClient;
    pub use unity_mirror_macro_rs::authenticator_factory;
}