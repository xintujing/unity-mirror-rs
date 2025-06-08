#![allow(dead_code)]
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use once_cell::sync::Lazy;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::{Deref, DerefMut};

pub enum TransportError {
    None,
    DnsResolve,       // 无法解析主机名
    Refused,          // 连接被另一端拒绝。服务器已满等
    Timeout,          // ping 超时或死链接
    Congestion,       // 消息数量超过传输/网络可以处理的数量
    InvalidReceive,   // 接收无效数据包（可能是故意攻击）
    InvalidSend,      // 用户尝试发送无效数据
    ConnectionClosed, // 连接自愿关闭或非自愿丢失
    Unexpected,       // 意外错误/异常，需要修复。
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum TransportChannel {
    Reliable = 1,
    Unreliable = 2,
}

impl Hash for TransportChannel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(*self as u8)
    }
}

impl Into<TransportChannel> for i32 {
    fn into(self) -> TransportChannel {
        match self {
            2 => TransportChannel::Unreliable,
            _ => TransportChannel::Reliable,
        }
    }
}

impl Display for TransportError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::DnsResolve => write!(f, "DNS 解析错误"),
            TransportError::Refused => write!(f, "连接被拒绝"),
            TransportError::Timeout => write!(f, "连接超时"),
            TransportError::Congestion => write!(f, "消息数量超过传输/网络可以处理的数量"),
            TransportError::InvalidReceive => write!(f, "接收无效数据包（可能是故意攻击）"),
            TransportError::InvalidSend => write!(f, "用户尝试发送无效数据"),
            TransportError::ConnectionClosed => write!(f, "连接自愿关闭"),
            TransportError::Unexpected => write!(f, "意外错误/异常，需要修复。"),
            TransportError::None => write!(f, ""),
        }
    }
}

static mut TRANSPORT_STATIC: Lazy<TransportStatic> = Lazy::new(|| TransportStatic { active: TransportStaticAction(None) });

pub struct TransportStaticAction(Option<RevelArc<Box<dyn Transport>>>);

impl From<RevelArc<Box<dyn Transport>>> for TransportStaticAction {
    fn from(value: RevelArc<Box<dyn Transport>>) -> Self {
        TransportStaticAction(Some(value))
    }
}

impl Deref for TransportStaticAction {
    type Target = RevelArc<Box<dyn Transport>>;
    fn deref(&self) -> &Self::Target {
        #[allow(static_mut_refs)]
        unsafe {
            self.0.as_ref().unwrap_or_else(|| {
                panic!("Transport not initialized. Call init_transport_manager first.")
            })
        }
    }
}

impl DerefMut for TransportStaticAction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(static_mut_refs)]
        unsafe {
            self.0.as_mut().unwrap_or_else(|| {
                panic!("Transport not initialized. Call init_transport_manager first.")
            })
        }
    }
}

pub struct TransportStatic {
    pub(crate) active: TransportStaticAction,
}

pub struct TransportManager;

impl Deref for TransportManager {
    type Target = TransportStatic;
    fn deref(&self) -> &Self::Target {
        #[allow(static_mut_refs)]
        unsafe {
            &TRANSPORT_STATIC
        }
    }
}

impl DerefMut for TransportManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(static_mut_refs)]
        unsafe {
            &mut TRANSPORT_STATIC
        }
    }
}

// impl TransportManager {
//     pub fn active(&self) -> &'static mut Box<dyn Transport> {
//         #[allow(static_mut_refs)]
//         unsafe {
//             TRANSPORT.as_mut().unwrap_or_else(|| {
//                 panic!("Transport not initialized. Call init_transport_manager first.")
//             })
//         }
//     }
// }

pub struct CallbackProcessor {
    pub on_server_connected: fn(u64),
    pub on_server_connected_with_address: fn(u64, &str),
    pub on_server_data_received: fn(u64, &[u8], TransportChannel),
    pub on_server_data_sent: fn(u64, &[u8], TransportChannel),
    pub on_server_error: fn(u64, TransportError, &str),
    pub on_server_transport_exception: fn(u64, Box<dyn std::error::Error>),
    pub on_server_disconnected: fn(u64),
}

pub trait Transport {
    fn init(&mut self, callback_processor: CallbackProcessor);
    /// <summary>此传输在当前平台可用吗？</summary>
    fn available(&self) -> bool;
    /// <summary>此传输是否已加密以实现安全通信？</summary>
    fn is_encrypted(&self) -> bool {
        false
    }
    /// <summary>如果加密，使用哪种密码？</summary>
    fn encryption_cipher(&self) -> String {
        "".to_string()
    }
    /// <summary>以 Uri 形式返回服务器地址。</summary>
    // 适用于 NetworkDiscovery。
    fn server_uri(&self) -> http::Uri;
    /// <summary>如果服务器当前正在监听连接，则为 True。</summary>
    fn server_active(&self) -> bool;
    /// <summary>开始监听连接。</summary>
    fn server_start(&mut self, _: (&str, u16));
    /// <summary>通过给定的渠道向客户端发送消息。</summary>
    fn server_send(&self, connection_id: u64, segment: &[u8], channel_id: TransportChannel);
    /// <summary>断开客户端与服务器的连接。</summary>
    fn server_disconnect(&self, connection_id: u64);
    /// <summary>获取服务器上的客户端地址。</summary>
    // 可用于游戏管理员 IP 禁令等。
    fn server_get_client_address(&self, connection_id: u64) -> Option<String>;
    /// <summary>停止监听并断开所有连接。</summary>
    fn server_stop(&self);
    /// <summary>给定通道的最大消息大小。</summary>
    // 不同的通道通常具有不同的大小，范围从 MTU 到几兆字节。
    // 需要始终返回一个值，即使传输未运行或可用，因为它需要进行初始化。
    fn get_max_packet_size(&self, channel_id: TransportChannel) -> usize;
    /// <summary>建议为此传输设置批处理阈值。</summary>
    // 默认使用 GetMaxPacketSize。
    // 某些传输（如 kcp）支持较大的最大数据包大小，但不应一直用于批处理，因为它们最终会变得太慢（队头阻塞等）。
    fn get_batch_threshold(&self, channel_id: TransportChannel) -> usize {
        self.get_max_packet_size(channel_id)
    }

    fn server_early_update(&self);
    fn server_late_update(&self);
    fn shutdown(&self);
    fn on_destroy(&self) {
        self.shutdown()
    }
}
