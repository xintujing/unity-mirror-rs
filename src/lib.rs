mod mirror;

use std::any::Any;

pub struct RemoteProcedureCalls;

pub(crate) static mut REGISTER_FUNCTIONS: Vec<
    fn(
        obj: Box<dyn NetworkBehaviour>,
        reader: NetworkReader,
        sender_connection: NetworkConnectionToClient,
    ),
> = vec![];

pub trait Type {}
pub trait RemoteCallDelegate {}

impl RemoteProcedureCalls {
    pub fn register_command(
        component_type: Option<()>,
        function_full_name: String,
        func: Option<
            fn(
                obj: Box<dyn NetworkBehaviour>,
                reader: NetworkReader,
                sender_connection: NetworkConnectionToClient,
            ),
        >,
        requires_authority: bool,
    ) {
        if let Some(_fn) = func {
            unsafe { REGISTER_FUNCTIONS.push(_fn) }
        }

        // func.unwrap()()
        // RegisterDelegate(
        //     component_type,
        //     function_full_name,
        //     RemoteCallType.Command,
        //     func,
        //     requires_authority,
        // );
    }
}

pub trait NetworkBehaviour {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct NetworkReader {
    data: Vec<u8>,
    position: usize,
}

impl NetworkReader {
    pub fn read<T>(&mut self) -> T {
        todo!()
    }
}

pub struct NetworkConnectionToClient {
    // network_connection: NetworkConnection,
    // pub reliable_rpcs_batch: NetworkWriter,
    // pub unreliable_rpcs_batch: NetworkWriter,
    // pub address: String,
    // pub observing: Vec<u32>,
    // pub drift_ema: ExponentialMovingAverage,
    // pub delivery_time_ema: ExponentialMovingAverage,
    // pub remote_timeline: f64,
    // pub remote_timescale: f64,
    // pub buffer_time_multiplier: f64,
    // pub buffer_time: f64,
    // pub snapshots: BTreeMap<OrderedFloat<f64>, TimeSnapshot>,
    // pub snapshot_buffer_size_limit: i32,
    // pub _rtt: ExponentialMovingAverage,
}
