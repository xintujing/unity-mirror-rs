use crate::commons::RevelArc;
use crate::commons::RevelWeak;
use crate::mirror::NetworkReader;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::{NetworkConnectionToClient, TNetworkBehaviour};
use once_cell::sync::Lazy;
use std::any::TypeId;
use std::collections::HashMap;

#[allow(unused)]
static mut REMOTE_CALL_DELEGATES: Lazy<HashMap<u16, Invoker>> = Lazy::new(|| HashMap::new());

#[allow(unused)]
pub type RemoteCallDelegate =
    fn(Vec<RevelWeak<Box<dyn TNetworkBehaviour>>>, &mut NetworkReader, RevelArc<Box<NetworkConnectionToClient>>);

#[allow(unused)]
pub struct RemoteProcedureCalls;

#[allow(unused)]
impl RemoteProcedureCalls {
    pub const INVOKE_RPC_PREFIX: &'static str = "InvokeUserCode_";

    pub fn register_command<T: TNetworkBehaviour + 'static>(
        &self,
        function_full_name: &str,
        func: RemoteCallDelegate,
        cmd_requires_authority: bool,
    ) -> u16 {
        // log::debug!("Registering remote procedure call: {}", function_full_name);

        self.register_delegate::<T>(
            function_full_name,
            RemoteCallType::Command,
            func,
            cmd_requires_authority,
        )
    }

    pub fn register_delegate<T: TNetworkBehaviour + 'static>(
        &self,
        function_full_name: &str,
        remote_call_type: RemoteCallType,
        func: RemoteCallDelegate,
        cmd_requires_authority: bool,
    ) -> u16 {
        let function_hash = function_full_name.fn_hash();
        let component_type = TypeId::of::<T>();

        if self.check_if_delegate_exists(
            component_type,
            function_full_name,
            &remote_call_type,
            func,
            &function_hash,
        ) {
            return function_hash;
        }

        #[allow(static_mut_refs)]
        unsafe {
            REMOTE_CALL_DELEGATES.insert(
                function_hash,
                Invoker {
                    component_type,
                    call_type: remote_call_type,
                    function: func,
                    cmd_requires_authority,
                    function_name: function_full_name.to_string(),
                },
            );
        }
        function_hash
    }

    fn check_if_delegate_exists(
        &self,
        component_type: TypeId,
        function_full_name: &str,
        remote_call_type: &RemoteCallType,
        func: RemoteCallDelegate,
        function_hash: &u16,
    ) -> bool {
        #[allow(static_mut_refs)]
        unsafe {
            if let Some(invoker) = REMOTE_CALL_DELEGATES.get(function_hash) {
                if invoker.are_equal(component_type, remote_call_type, func) {
                    return true;
                }

                log::error!("Function {:?}.{}, and {:?}.{}, have the same hash. Please rename one of them. To save bandwidth, we only use 2 bytes for the hash, which has a small chance of collisions.",
                    invoker.component_type,
                    invoker.function_name,
                    component_type,
                    function_full_name);
            }
        }
        false
    }

    pub fn invoke(
        &self,
        function_hash: u16,
        remote_call_type: &RemoteCallType,
        reader: &mut NetworkReader,
        network_behaviour_chain: Vec<RevelWeak<Box<dyn TNetworkBehaviour>>>,
        conn: RevelArc<Box<NetworkConnectionToClient>>,
    ) -> bool {
        if let Some(invoker) = self.get_invoker_for_hash(function_hash, remote_call_type) {
            (invoker.function)(network_behaviour_chain, reader, conn);
            return true;
        }
        false
    }

    fn get_invoker_for_hash(
        &self,
        function_hash: u16,
        remote_call_type: &RemoteCallType,
    ) -> Option<&'static Invoker> {
        #[allow(static_mut_refs)]
        unsafe {
            if let Some(invoker) = REMOTE_CALL_DELEGATES.get(&function_hash) {
                if invoker.call_type == *remote_call_type {
                    return Some(invoker);
                }
            }
        }
        None
    }

    pub fn get_function_method_name(&self, function_hash: u16) -> Option<String> {
        #[allow(static_mut_refs)]
        unsafe {
            if let Some(invoker) = REMOTE_CALL_DELEGATES.get(&function_hash) {
                return Some(invoker.function_name.clone());
            }
        }
        None
    }

    pub fn command_requires_authority(&self, function_hash: &u16) -> bool {
        #[allow(static_mut_refs)]
        unsafe {
            if let Some(invoker) = REMOTE_CALL_DELEGATES.get(function_hash) {
                return invoker.cmd_requires_authority;
            }
        }
        false
    }

    pub fn remove_delegate(&self, function_hash: u16) {
        #[allow(static_mut_refs)]
        unsafe {
            REMOTE_CALL_DELEGATES.remove(&function_hash);
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum RemoteCallType {
    #[allow(unused)]
    Command,
    #[allow(unused)]
    ClientRpc,
}

pub struct Invoker {
    pub(crate) component_type: TypeId,
    pub(crate) call_type: RemoteCallType,
    pub(crate) function: RemoteCallDelegate,
    #[allow(unused)]
    pub(crate) cmd_requires_authority: bool,
    pub(crate) function_name: String,
}

impl Invoker {
    fn are_equal(
        &self,
        component_type: TypeId,
        call_type: &RemoteCallType,
        invoke_function: RemoteCallDelegate,
    ) -> bool {
        self.component_type == component_type
            && &self.call_type == call_type
            && std::ptr::fn_addr_eq(self.function, invoke_function)
    }
}
