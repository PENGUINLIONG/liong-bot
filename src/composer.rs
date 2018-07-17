use super::Msg;
use failure::Error;

pub trait Composer {
    fn name(&self) -> &'static str;
    fn compose(&self, msg: &Msg) -> Result<String, Error>;
    fn decompose(&self, raw: &str) -> Result<Msg, Error>;
}
