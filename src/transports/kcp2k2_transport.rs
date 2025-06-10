use crate::mirror::transport::{CallbackProcessor, Transport, TransportChannel, TransportError};
use http::Uri;
use kcp2k_rust::error_code::ErrorCode;
use kcp2k_rust::kcp2k::Kcp2K;
use kcp2k_rust::kcp2k_callback::{Callback, CallbackType};
use kcp2k_rust::kcp2k_channel::Kcp2KChannel;
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use kcp2k_rust::kcp2k_connection::Kcp2KConnection;
use kcp2k_rust::kcp2k_peer::Kcp2KPeer;
use std::net::ToSocketAddrs;
use std::process::exit;
use std::str::FromStr;
use unity_mirror_macro_rs::CallbackProcessor;

const SCHEMA: &str = "kcp";

#[derive(Default, CallbackProcessor)]
pub struct Kcp2kTransport {
    pub server_active: bool,
    pub config: Kcp2KConfig,
    pub port: u16,
    pub kcp_serv: Option<Kcp2K>,
}
impl Kcp2kTransport {
    pub fn new(config: Option<Kcp2KConfig>) -> Box<Self> {
        Box::new(Self {
            server_active: false,
            config: if config.is_some() {
                config.unwrap()
            } else {
                Default::default()
            },
            port: 0,
            kcp_serv: None,
        })
    }

    pub fn kcp2k_callback(conn: &Kcp2KConnection, c: Callback) {
        match c.r#type {
            CallbackType::OnConnected => on_server_connected(conn.get_connection_id()),
            CallbackType::OnData => {
                on_server_data_received(conn.get_connection_id(), c.data.as_ref(), c.channel.into())
            }
            CallbackType::OnDisconnected => {
                on_server_disconnected(conn.get_connection_id());
            }
            CallbackType::OnError => {
                on_server_error(
                    conn.get_connection_id(),
                    c.error_code.into(),
                    &c.error_message,
                );
            }
        }
    }
}

#[allow(unused)]
impl Transport for Kcp2kTransport {
    fn init(&mut self, callback_processor: CallbackProcessor) {
        init_kcp2k_transport_callback_processor(callback_processor)
    }

    fn available(&self) -> bool {
        true
    }

    fn server_uri(&self) -> Uri {
        let host = hostname::get().ok().unwrap().into_string().ok().unwrap();

        let addr = format!("{}:{}", host, self.port);
        let socket_addr = addr.to_socket_addrs().ok().unwrap().next().unwrap();

        let uri_str = format!("{}://{}:{}", SCHEMA, socket_addr.ip(), socket_addr.port());
        Uri::from_str(&uri_str).ok().unwrap()
    }

    fn server_active(&self) -> bool {
        self.server_active
    }

    fn server_start(&mut self, (mut network_address, port): (&str, u16)) {
        self.port = port;
        match Kcp2K::new_server(
            self.config,
            format!(
                "{}:{}",
                network_address.replace("localhost", "0.0.0.0"),
                self.port
            ),
            Self::kcp2k_callback,
        ) {
            Ok(server) => {
                self.kcp_serv = Some(server);
                self.server_active = true;
            }
            Err(err) => {
                log::error!("Kcp2kTransport awake error: {:?}", err);
                exit(1)
            }
        }
    }

    fn server_send(&self, connection_id: u64, segment: &[u8], channel_id: TransportChannel) {
        if let Some(serv) = &self.kcp_serv {
            match serv.s_send(
                connection_id,
                bytes::Bytes::copy_from_slice(segment),
                channel_id.into(),
            ) {
                Ok(_) => {
                    on_server_data_sent(connection_id, segment, TransportChannel::from(channel_id))
                }
                Err(err_code) => {
                    let reason = format!("{:?}", err_code);
                    on_server_error(connection_id, err_code.into(), &reason)
                }
            }
        };
    }

    fn server_disconnect(&self, connection_id: u64) {
        if let Some(serv) = &self.kcp_serv {
            serv.close_connection(connection_id)
        }
    }

    fn server_get_client_address(&self, connection_id: u64) -> Option<String> {
        if let Some(serv) = &self.kcp_serv {
            Some(serv.get_connection_address(connection_id))
        } else {
            None
        }
    }

    fn server_stop(&self) {
        if let Some(serv) = &self.kcp_serv {
            if let Err(err) = serv.stop() {
                println!("Error stopping server: {}", err)
            };
        }
    }

    fn get_max_packet_size(&self, channel_id: TransportChannel) -> usize {
        match channel_id {
            TransportChannel::Reliable => {
                Kcp2KPeer::unreliable_max_message_size(self.config.mtu as u32)
            }
            TransportChannel::Unreliable => Kcp2KPeer::reliable_max_message_size(
                self.config.mtu as u32,
                self.config.receive_window_size as u32,
            ),
        }
    }

    #[allow(unused)]
    fn get_batch_threshold(&self, channel_id: TransportChannel) -> usize {
        Kcp2KPeer::unreliable_max_message_size(self.config.mtu as u32)
    }

    fn server_early_update(&self) {
        if let Some(serv) = &self.kcp_serv {
            serv.tick_incoming()
        }
    }

    fn server_late_update(&self) {
        if let Some(serv) = &self.kcp_serv {
            serv.tick_outgoing()
        }
    }

    fn shutdown(&self) {}
}

impl Into<TransportChannel> for Kcp2KChannel {
    fn into(self) -> TransportChannel {
        match self {
            Kcp2KChannel::None => TransportChannel::Reliable,
            Kcp2KChannel::Reliable => TransportChannel::Reliable,
            Kcp2KChannel::Unreliable => TransportChannel::Unreliable,
        }
    }
}

impl Into<Kcp2KChannel> for TransportChannel {
    fn into(self) -> Kcp2KChannel {
        match self {
            TransportChannel::Reliable => Kcp2KChannel::Reliable,
            TransportChannel::Unreliable => Kcp2KChannel::Unreliable,
        }
    }
}

impl Into<TransportError> for ErrorCode {
    fn into(self) -> TransportError {
        match self {
            ErrorCode::None => TransportError::None,
            ErrorCode::DnsResolve => TransportError::DnsResolve,
            ErrorCode::Timeout => TransportError::Timeout,
            ErrorCode::Congestion => TransportError::Congestion,
            ErrorCode::InvalidReceive => TransportError::InvalidReceive,
            ErrorCode::InvalidSend | ErrorCode::SendError => TransportError::InvalidSend,
            ErrorCode::ConnectionClosed => TransportError::ConnectionClosed,
            ErrorCode::Unexpected => TransportError::Unexpected,
            ErrorCode::ConnectionNotFound | ErrorCode::ConnectionLocked => {
                TransportError::Unexpected
            }
        }
    }
}
