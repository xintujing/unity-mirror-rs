use crate::mirror::core::network_behaviour::NetworkBehaviourType;
use crate::mirror::core::tools::stable_hash::StableHash;
use crate::{log_error, NetworkConnectionToClient, NetworkReader};
use dashmap::DashMap;
use lazy_static::lazy_static;
use std::any::TypeId;

lazy_static! {
    static ref NETWORK_MESSAGE_HANDLERS: DashMap<u16, Invoker> = DashMap::new();
}

pub struct RemoteProcedureCalls;

impl RemoteProcedureCalls {
    pub fn register_command_delegate<T: 'static>(
        function_full_name: &str,
        func: RemoteCallDelegate,
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
        func: RemoteCallDelegate,
    ) -> u16 {
        Self::register_delegate::<T>(function_full_name, RemoteCallType::ClientRpc, func, true)
    }
    pub fn register_delegate<T: 'static>(
        function_full_name: &str,
        remote_call_type: RemoteCallType,
        func: RemoteCallDelegate,
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
        func: &RemoteCallDelegate,
        func_hash: u16,
    ) -> bool {
        if let Some(old_invoker) = NETWORK_MESSAGE_HANDLERS.get(&func_hash) {
            if old_invoker.are_equal(type_id, remote_call_type, func) {
                return true;
            }
            log_error!("Delegate already exists for hash: {}", func_hash);
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
pub type RemoteCallDelegate =
    fn(obj: NetworkBehaviourType, reader: NetworkReader, connection: NetworkConnectionToClient);

#[derive(Debug)]
struct Invoker {
    pub type_id: TypeId,
    pub call_type: RemoteCallType,
    pub function: RemoteCallDelegate,
    pub requires_authority: bool,
}

impl Invoker {
    pub fn new(
        type_id: TypeId,
        call_type: RemoteCallType,
        function: RemoteCallDelegate,
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
        invoke_function: &RemoteCallDelegate,
    ) -> bool {
        self.type_id == type_id
            && self.call_type == remote_call_type
            && self.function.eq(invoke_function)
    }
}
