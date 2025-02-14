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
pub struct Transport {
    pub transport_cb_fn: TransportCallBackFuncType,
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            transport_cb_fn: |_: TransportCallback| {},
        }
    }
}
