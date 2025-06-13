use crate::commons::action::SelfMutAction;
use crate::commons::RevelArc;
use crate::commons::RevelWeak;
use crate::mirror::NetworkConnectionToClient;
use crate::unity_engine::MonoBehaviour;


pub trait Authenticator: AuthenticatorBase + MonoBehaviour {
    fn new() -> Self
    where
        Self: Sized;
    fn on_start_server(&self) {}
    fn on_stop_server(&self) {}
    fn server_accept(&self, connection: RevelArc<Box<NetworkConnectionToClient>>) {
        self.on_server_authenticated().call((connection,));
    }
    fn on_server_authenticate(&self, connection: RevelArc<Box<NetworkConnectionToClient>>) {}
    fn server_reject(&self, conn: &mut NetworkConnectionToClient) {
        conn.disconnect.call(());
    }
}

pub trait AuthenticatorBase {
    fn set_weak_self(&mut self, weak_self: RevelWeak<Box<dyn Authenticator>>);
    fn set_on_server_authenticated(&mut self, event: SelfMutAction<(RevelArc<Box<NetworkConnectionToClient>>,), ()>);
    fn on_server_authenticated(&self) -> &SelfMutAction<(RevelArc<Box<NetworkConnectionToClient>>,), ()>;
}
