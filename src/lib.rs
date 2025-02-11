// mod mirror;
// mod unity;
//
// mod demos;

pub struct RemoteProcedureCalls;

pub(crate) static mut REGISTER_FUNCTIONS: Vec<fn()> = vec![];

pub trait Type {}
pub trait RemoteCallDelegate {}

impl RemoteProcedureCalls {
    pub fn register_command(
        component_type: Option<()>,
        function_full_name: String,
        func: Option<fn()>,
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

pub fn aaa() {
    unsafe {
        for x in REGISTER_FUNCTIONS.iter() {
            x();
        }
    }
}
