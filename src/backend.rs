use super::Msg;

pub struct BackendMetadata {
    pub identity: &'static str,
    pub name: &'static str,
    pub author: &'static str,
    pub version: &'static str,
    pub description: &'static str,
}

pub trait Backend {
    fn metadata(&self) -> BackendMetadata;
    fn preview(&self, msg: Msg) -> Msg;
    /// Process message and give a response.
    fn process(&self, msg: Msg) -> Msg;
}
