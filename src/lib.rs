extern crate base64;
extern crate bytes;
extern crate encoding;
#[macro_use]
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate structopt;
extern crate dotenv;
extern crate failure;

mod backend;
mod composer;
mod dispatcher;
mod msg;
mod peripheral;
pub mod sys;

use backend::Backend;
use composer::Composer;
use dispatcher::Dispatcher;
use msg::Msg;

pub fn on_launch() {
}
pub fn on_shutdown() {

}
pub fn on_configure(dispatcher: &mut Dispatcher) {

}
pub fn on_recv_priv(qq: i64, raw: &str) {
}
pub fn on_recv_grp(grp: i64, qq: i64, raw: &str) {
}
