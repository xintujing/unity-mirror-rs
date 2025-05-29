use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::NetworkBehaviourT;
use once_cell::sync::Lazy;
use std::any::TypeId;
use std::collections::HashMap;

static mut REMOTE_CALL_DELEGATES: Lazy<HashMap<u16, Invoker>> = Lazy::new(|| HashMap::new());

pub struct RemoteProcedureCalls;

impl RemoteProcedureCalls {
    pub const INVOKE_RPC_PREFIX: &'static str = "InvokeUserCode_";

    pub fn register_delegate<T: NetworkBehaviourT + 'static>(
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

    pub fn check_if_delegate_exists(
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
}

#[derive(Eq, PartialEq)]
pub enum RemoteCallType {
    #[allow(unused)]
    Command,
    #[allow(unused)]
    ClientRpc,
}
pub type RemoteCallDelegate = fn(
    &Vec<RevelWeak<Box<dyn NetworkBehaviourT>>>,
    &mut NetworkReader,
    &mut RevelArc<NetworkConnection>,
);

pub struct Invoker {
    pub(crate) component_type: TypeId,
    pub(crate) call_type: RemoteCallType,
    pub(crate) function: RemoteCallDelegate,
    pub(crate) cmd_requires_authority: bool,
    pub(crate) function_name: String,
}

impl Invoker {
    pub fn are_equal(
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
