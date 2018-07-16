use super::Msg;

pub trait Composer {
    fn name(&self) -> &'static str;
    fn compose(&self, msg: Msg) -> String;
    fn decompose(&self, raw: String) -> Msg;
}
