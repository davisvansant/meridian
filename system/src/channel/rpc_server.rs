use tokio::sync::mpsc;

pub type RpcServerShutdown = mpsc::Receiver<bool>;
