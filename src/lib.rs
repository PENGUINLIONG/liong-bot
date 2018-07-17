extern crate base64;
extern crate bytes;
extern crate encoding_rs;
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
#[macro_use]
mod msg;
mod peripheral;
pub mod sys;

use backend::Backend;
use composer::Composer;
use dispatcher::Dispatcher;
use msg::{Msg, MsgIn};

pub fn on_launch() {
}
pub fn on_shutdown() {

}
pub fn on_configure(dispatcher: &mut Dispatcher) {
    use peripheral::coolq::CoolQComposer;

    let local = std::env::current_dir().unwrap();
    dispatcher
        .use_composer(CoolQComposer::new(&local));
}
