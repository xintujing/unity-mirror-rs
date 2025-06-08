use crate::commons::action::SelfMutAction;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::mirror::batching::batcher::Batcher;
use crate::mirror::batching::un_batcher::UnBatcher;
use crate::mirror::messages::message;
use crate::mirror::messages::message::NetworkMessage;
use crate::mirror::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::mirror::transport::{TransportChannel, TransportManager};
use crate::mirror::NetworkWriterPool;
use crate::mirror::{NetworkConnectionToClient, NetworkIdentity};
use crate::mirror::{NetworkTime, NetworkWriter};
use crate::unity_engine::{ExponentialMovingAverage, Time};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use unity_mirror_macro_rs::{action, internal, virtual_shroud, virtual_trait};

#[derive(Default)]
pub struct NetworkConnection {
    pub self_weak: RevelWeak<Box<NetworkConnection>>,

    local_connection_id: i32,
    /// <summary>由传输层分配的此连接的唯一标识符。</summary>
    pub connection_id: u64,
    /// <summary>指示客户端已进行身份验证的标志。</summary>
    pub is_authenticated: bool,
    /// <summary>加入游戏世界后准备就绪服务器连接。</summary>
    pub is_ready: bool,
    /// <summary>上次收到有关此连接的消息。包括系统和用户消息。</summary>
    pub last_message_time: f32,
    /// <summary>此连接的主要对象（通常是播放器对象）。</summary>
    pub identity: RevelWeak<Box<NetworkIdentity>>,
    /// <summary>此连接拥有的所有网络认同。可以是主要玩家，宠物等</summary>
    pub owned: HashSet<RevelArc<Box<NetworkIdentity>>>,
    /// <summary>从服务器到客户端和客户端到服务器的可靠批处理。</summary>
    pub reliable_batcher: Option<RevelArc<Batcher>>,
    /// <summary>从服务器到客户端和客户端到服务器的非可靠批处理。</summary>
    pub unreliable_batcher: Option<RevelArc<Batcher>>,
    /// <summary>最后一批的远程时间戳。未插值。对NetworkTransForm等有用。</summary>
    pub remote_time_stamp: f64,

    // Actions
    pub update: SelfMutAction<(), ()>,
    pub send_to_transport: SelfMutAction<(Vec<u8>, TransportChannel), ()>,
    pub is_alive: SelfMutAction<(f32,), bool>,
    pub disconnect: SelfMutAction<(), ()>,
    pub cleanup: SelfMutAction<(), ()>,
}

impl NetworkConnection {
    pub fn new(connection_id: u64) -> RevelArc<Box<Self>> {
        let mut connection = RevelArc::new(Box::new(Self::default()));
        connection.connection_id = connection_id;
        connection.self_weak = connection.downgrade();
        connection
    }
}

// 生命周期
impl NetworkConnection {
    //在每个更新结束时冲洗批处理的消息。
    // #[virtual_shroud]
    #[action]
    pub fn update(&mut self) {
        let writer = NetworkWriter::new();

        NetworkWriterPool::get_by_closure(|writer| {
            if let Some(reliable_batcher) = &mut self.reliable_batcher {
                reliable_batcher.get_batcher_writer(writer);
                self.send_to_transport
                    .call((writer.to_vec(), TransportChannel::Reliable));
            }
        });

        NetworkWriterPool::get_by_closure(|writer| {
            if let Some(unreliable_batcher) = &mut self.unreliable_batcher {
                unreliable_batcher.get_batcher_writer(writer);
                self.send_to_transport
                    .call((writer.to_vec(), TransportChannel::Unreliable));
            }
        });
    }
}

impl NetworkConnection {
    fn get_batch_for_channel_id(&mut self, channel_id: TransportChannel) -> RevelArc<Batcher> {
        match channel_id {
            TransportChannel::Reliable => match &self.reliable_batcher {
                None => {
                    let threshold = TransportManager.active.get_batch_threshold(channel_id);
                    let reliable_batcher = RevelArc::new(Batcher::new(threshold));
                    self.reliable_batcher = Some(reliable_batcher.clone());
                    reliable_batcher
                }
                Some(reliable_batcher) => reliable_batcher.clone(),
            },
            TransportChannel::Unreliable => match &self.unreliable_batcher {
                None => {
                    let threshold = TransportManager.active.get_batch_threshold(channel_id);
                    let unreliable_batcher = RevelArc::new(Batcher::new(threshold));
                    self.unreliable_batcher = Some(unreliable_batcher.clone());
                    unreliable_batcher
                }
                Some(unreliable_batcher) => unreliable_batcher.clone(),
            },
        }
    }

    /// <summary>通过给定的频道向此连接发送一个网络。</summary>
    pub fn send_message<T>(&mut self, mut message: T, channel_id: TransportChannel)
    where
        T: NetworkMessage,
    {
        NetworkWriterPool::get_by_closure(|writer| {
            message.serialize(writer);
            let max_size = message::max_message_size(channel_id);
            if writer.position > max_size {
                log::error!(
                    "NetworkServer.SendToAll: message of type {} with a size of {} bytes is larger than the max allowed message size in one batch: {}.\nThe message was dropped, please make it smaller.",
                    T::get_full_name(),
                    writer.position,
                    max_size
                );
                return;
            }

            self.send(writer.to_slice(), channel_id);
        });
    }

    //发送第二阶段：序列化网络作为阵列<byte>
    //内部，因为除了镜子外，没有人应直接发送字节
    //客户。它们将被检测为消息。而是发送消息。
    // =>在调用发送<byte>之前，请确保验证消息<t>大小！
    pub(crate) fn send(&mut self, segment: &[u8], channel_id: TransportChannel) {
        // match channel_id {
        //     TransportChannel::Reliable => {
        //         if let Some(reliable_batcher) = &mut self.reliable_batcher {
        //             reliable_batcher.add_message(segment, NetworkTime.local_time())
        //         }
        //     }
        //     TransportChannel::Unreliable => {
        //         if let Some(unreliable_batcher) = &mut self.unreliable_batcher {
        //             unreliable_batcher.add_message(segment, NetworkTime.local_time())
        //         }
        //     }
        // }

        self.get_batch_for_channel_id(channel_id)
            .add_message(segment, NetworkTime.local_time())
    }
    /// <summary>检查我们是否在最后一个“超时”秒内收到了一条消息。</summary>
    #[action]
    pub fn is_alive(&self, timeout: f32) -> bool {
        Time::unscaled_time_f64() - (self.last_message_time as f64) < timeout as f64
    }

    #[action]
    pub fn cleanup(&mut self) {
        if let Some(reliable_batcher) = &mut self.reliable_batcher {
            reliable_batcher.clear()
        }
        if let Some(unreliable_batcher) = &mut self.unreliable_batcher {
            unreliable_batcher.clear()
        }
    }
}

impl Display for NetworkConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "connection({})", self.connection_id)
    }
}
