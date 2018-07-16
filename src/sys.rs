//! Import symbols from `CQP.dll`
use std::cell::Cell;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use encoding::all::GB18030;
use encoding::{DecoderTrap, EncoderTrap, Encoding};
use ::Dispatcher;

mod consts {
    pub const APP_INFO: &'static str = "9,moe.penguinliong.liongbot\0";

    pub const EVENT_IGNORE: i32 = 0;
    pub const EVENT_BLOCK: i32 = 1;
}

static mut DISPATCHER: Option<Dispatcher> = None;
static mut AUTH: i32 = 0;

pub fn send_priv(qq: i64, msg: &str) {
    #[no_mangle]
    #[link(name="CQP")]
    extern {
        #[link_name="CQ_sendPrivateMsg"]
        fn native(auth: i32, qq: i64, msg: *const c_char) -> i32;
    }
    let buf = GB18030.encode(msg, EncoderTrap::Replace).unwrap();
    // FIXME: (penguinliong) Check result.
    let _ = unsafe { native(AUTH, qq, CString::new(buf).unwrap().as_ptr()) };
}
pub fn send_grp(grp: i64, qq: i64, msg: &str) {
    #[no_mangle]
    #[link(name="CQP")]
    extern {
        #[link_name="CQ_sendGroupMsg"]
        fn native(auth: i32, grp: i64, msg: *const c_char) -> i32;
    }
    let buf = GB18030.encode(msg, EncoderTrap::Replace).unwrap();
    // FIXME: (penguinliong) Check result.
    let _ = unsafe { native(AUTH, grp, CString::new(buf).unwrap().as_ptr()); };
}
pub fn get_user_info(qq: i64) {
    #[no_mangle]
    #[link(name="CQP")]
    extern {
        #[link_name="CQ_getStrangerInfo"]
        fn native(auth: i32, qq: i64, no_cache: i32) -> *const u8;
    }
}
pub fn get_grp_member_info(grp: i64, qq: i64) {
    #[no_mangle]
    #[link(name="CQP")]
    extern {
        #[link_name="CQ_getGroupMemberInfoV2"]
        fn native (auth: i32, grp: i64, qq: i64, no_cache: i32) -> *const u8;
    }
}

#[no_mangle]
#[export_name = "AppInfo"]
pub extern "stdcall" fn native_app() -> *const u8 {
    consts::APP_INFO.as_ptr()
}
#[no_mangle]
#[export_name = "Initialize"]
pub extern "stdcall" fn native_init(auth: i32) -> i32 {
    unsafe { AUTH = auth; }
    0
}
#[no_mangle]
pub extern "stdcall" fn native_launch() -> i32 {
    ::on_launch();
    let mut d = Dispatcher::new();
    ::on_configure(&mut d);
    unsafe { DISPATCHER = Some(d); }
    0
}
#[no_mangle]
pub extern "stdcall" fn native_shutdown() -> i32 {
    ::on_shutdown();
    0
}
#[no_mangle]
pub extern "stdcall" fn native_enable() -> i32 {
    0
}
#[no_mangle]
pub extern "stdcall" fn native_disable() -> i32 {
    0
}
#[no_mangle]
pub extern "stdcall" fn native_on_recv_priv(subtype: i32,
                                  msgId: i32,
                                  from_qq: i64,
                                  msg: *const c_char,
                                  font: i32) -> i32 {
    let raw = unsafe { CStr::from_ptr(msg) };
    let decoded = GB18030.decode(raw.to_bytes(), DecoderTrap::Replace).unwrap();
    ::on_recv_priv(from_qq, &decoded);
    consts::EVENT_IGNORE
}
#[no_mangle]
pub extern "stdcall" fn native_on_recv_grp(subtype: i32,
                                 msgId: i32,
                                 from_grp: i64,
                                 from_qq: i64,
                                 from_anon: *const c_char,
                                 msg: *const c_char,
                                 font: i32) -> i32 {

    // Ignore anonymous instructions.
    if !from_anon.is_null() { return consts::EVENT_IGNORE }
    let raw = unsafe { CStr::from_ptr(msg) };
    let decoded = GB18030.decode(raw.to_bytes(), DecoderTrap::Replace).unwrap();
    ::on_recv_grp(from_grp, from_qq, &decoded);
    consts::EVENT_IGNORE
}
