#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum TransportChannel {
    Unreliable = 0,
    Reliable = 1,
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(u8)]
pub enum TransportCallback {
    OnConnected(u64, TransportChannel),
    OnSent(u64, Vec<u8>, TransportChannel),
    OnReceived(u64, Vec<u8>, TransportChannel),
    OnDisconnected(u64, TransportChannel),
    OnError(u64, TransportError, TransportChannel),
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(u8)]
pub enum TransportError {
    DnsResolve(String),         // failed to resolve a host name
    Refused(String),            // connection refused by other end. server full etc.
    Timeout(String),            // ping timeout or dead link
    Congestion(String),         // more messages than transport / network can process
    ReceiveInvalid(String),     // recv invalid packet (possibly intentional attack)
    SendInvalidData(String),    // user tried to send invalid data
    ConnectionClosed(String),   // connection closed voluntarily or lost involuntarily
    SendDataError(String),      // failed to send data
    ConnectionNotFound(String), // connection not found
    ConnectionLocked(String),   // connection is locked
    Unexpected(String),         // unexpected error / exception, requires fix.
}

// 传输回调函数类型
pub type TransportCallBackFuncType = fn(TransportCallback);

// 传输层抽象
#[derive(Default)]
pub struct Transport {
    pub transport_cb_fn: Option<TransportCallBackFuncType>,
}

pub trait TransportTrait {
    fn get_client_address(&self, conn_id: u64) -> String;
    fn send(&self, conn_id: u64, data: Vec<u8>, channel: TransportChannel);
    fn disconnect(&self, conn_id: u64);
    fn transport_cb_fn(&self) -> Option<TransportCallBackFuncType>;
    fn set_transport_cb_fn(&mut self, cb: TransportCallBackFuncType);
    fn early_update(&mut self);
    fn late_update(&mut self);
}
