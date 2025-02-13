use crate::mirror::core::network_behaviour::NetworkBehaviourType;
use crate::mirror::core::network_connection_to_client::NetworkConnectionToClient;
use crate::mirror::core::tools::stable_hash::StableHash;
use crate::{log_error, NetworkReader};
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use lazy_static::lazy_static;
use std::any::TypeId;

lazy_static! {
    static ref NETWORK_MESSAGE_HANDLERS: DashMap<u16, Invoker> = DashMap::new();
}

pub struct RemoteProcedureCalls;

impl RemoteProcedureCalls {
    pub fn invoke(
        func_hash: u16,
        remote_call_type: RemoteCallType,
        reader: &mut NetworkReader,
        obj: &mut NetworkBehaviourType,
        connection_to_client: &mut NetworkConnectionToClient,
    ) -> bool {
        match Self::get_invoker_for_hash(func_hash, remote_call_type) {
            // 没有找到对应的委托
            None => false,
            // 找到对应的委托
            Some(invoker) => {
                (invoker.function)(obj, reader, connection_to_client);
                true
            }
        }
    }

    pub fn command_requires_authority(func_hash: u16) -> bool {
        if let Some(invoker) = NETWORK_MESSAGE_HANDLERS.get(&func_hash) {
            return invoker.requires_authority;
        }
        false
    }

    fn get_invoker_for_hash(
        func_hash: u16,
        remote_call_type: RemoteCallType,
    ) -> Option<Ref<'static, u16, Invoker>> {
        if let Some(invoker) = NETWORK_MESSAGE_HANDLERS.get(&func_hash) {
            if invoker.call_type == remote_call_type {
                return Some(invoker);
            }
        }
        None
    }

    pub fn register_command_delegate<T: 'static>(
        function_full_name: &str,
        func: RemoteCallDelegateType,
        cmd_requires_authority: bool,
    ) -> u16 {
        Self::register_delegate::<T>(
            function_full_name,
            RemoteCallType::Command,
            func,
            cmd_requires_authority,
        )
    }

    pub fn register_rpc_delegate<T: 'static>(
        function_full_name: &str,
        func: RemoteCallDelegateType,
    ) -> u16 {
        Self::register_delegate::<T>(function_full_name, RemoteCallType::ClientRpc, func, true)
    }
    pub fn register_delegate<T: 'static>(
        function_full_name: &str,
        remote_call_type: RemoteCallType,
        func: RemoteCallDelegateType,
        cmd_requires_authority: bool,
    ) -> u16 {
        let hash = function_full_name.get_fn_stable_hash_code();
        let type_id = Self::generate_type_id::<T>();
        if Self::check_if_delegate_exists(type_id, remote_call_type, &func, hash) {
            return hash;
        }
        let invoker = Invoker::new(type_id, remote_call_type, func, cmd_requires_authority);
        NETWORK_MESSAGE_HANDLERS.insert(hash, invoker);
        hash
    }

    fn check_if_delegate_exists(
        type_id: TypeId,
        remote_call_type: RemoteCallType,
        func: &RemoteCallDelegateType,
        func_hash: u16,
    ) -> bool {
        if let Some(old_invoker) = NETWORK_MESSAGE_HANDLERS.get(&func_hash) {
            if old_invoker.are_equal(type_id, remote_call_type, func) {
                return true;
            }
            log_error!(format!("Delegate already exists for hash: {}", func_hash));
        }
        false
    }

    pub fn generate_type_id<T: 'static>() -> TypeId {
        TypeId::of::<T>()
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum RemoteCallType {
    Command,
    ClientRpc,
}

// RemoteCallDelegate is a function pointer type
pub type RemoteCallDelegateType = fn(
    obj: &mut NetworkBehaviourType,
    reader: &mut NetworkReader,
    connection: &mut NetworkConnectionToClient,
);

#[derive(Debug)]
struct Invoker {
    pub type_id: TypeId,
    pub call_type: RemoteCallType,
    pub function: RemoteCallDelegateType,
    pub requires_authority: bool,
}

impl Invoker {
    pub fn new(
        type_id: TypeId,
        call_type: RemoteCallType,
        function: RemoteCallDelegateType,
        requires_authority: bool,
    ) -> Self {
        Invoker {
            type_id,
            call_type,
            function,
            requires_authority,
        }
    }

    pub fn are_equal(
        &self,
        type_id: TypeId,
        remote_call_type: RemoteCallType,
        invoke_function: &RemoteCallDelegateType,
    ) -> bool {
        self.type_id == type_id
            && self.call_type == remote_call_type
            && self.function.eq(invoke_function)
    }
}
