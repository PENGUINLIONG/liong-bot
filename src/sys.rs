//! Import symbols from `CQP.dll`
use std::cell::Cell;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use encoding_rs::GB18030;
use base64::decode;
use bytes::{Buf, IntoBuf, Bytes};
use {Dispatcher, Msg, MsgIn};

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
    let (buf, _, _) = GB18030.encode(msg);
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
    // FIXME: (penguinliong) Check result.
    let (buf, _, _) = GB18030.encode(msg);
    let _ = unsafe { native(AUTH, grp, CString::new(buf).unwrap().as_ptr()); };
}
pub fn skip_string(b: &mut Buf) {
    loop {
        match b.get_u8() {
            0 => break,
            _ => continue,
        }
    }
}
pub fn get_string(b: &mut Buf) -> String {
    let mut buf = Vec::new();
    loop {
        match b.get_u8() {
            0 => break,
            c => buf.push(c),
        }
    }
    // FIXME: (penguinliong) Check result.
    use encoding_rs::GB18030;
    let (rv, _) = GB18030.decode_without_bom_handling(&buf);
    String::from(rv)
}
pub fn make_priv_msg_in(qq: i64, content: Msg) -> MsgIn {
    #[no_mangle]
    #[link(name="CQP")]
    extern {
        #[link_name="CQ_getStrangerInfo"]
        fn native(auth: i32, qq: i64, no_cache: i32) -> *const c_char;
    }
    let b64 = unsafe { CStr::from_ptr(native(AUTH, qq, 0)) };
    let raw = decode(b64.to_bytes()).unwrap();
    let mut b = Bytes::from(raw).into_buf();

    b.advance(8);
    let alias = get_string(&mut b);
    MsgIn::Private {
        qq: qq,
        alias: alias,
        content: content,
    }
}
pub fn make_grp_msg_in(grp: i64, qq: i64, content: Msg) -> MsgIn {
    #[no_mangle]
    #[link(name="CQP")]
    extern {
        #[link_name="CQ_getGroupMemberInfoV2"]
        fn native (auth: i32, grp: i64, qq: i64, no_cache: i32)
            -> *const c_char;
    }
    let b64 = unsafe { CStr::from_ptr(native(AUTH, grp, qq, 0)) };
    let raw = decode(b64.to_bytes()).unwrap();
    let mut b = Bytes::from(raw).into_buf();

    b.advance(16);
    let alias = get_string(&mut b);
    let grp_alias = get_string(&mut b);
    MsgIn::Group {
        grp: grp,
        qq: qq,
        alias: alias,
        grp_alias: grp_alias,
        content: content,
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
    let mut dispatcher = Dispatcher::new();
    ::on_configure(&mut dispatcher);
    unsafe { DISPATCHER = Some(dispatcher); }
    0
}
#[no_mangle]
pub extern "stdcall" fn native_shutdown() -> i32 {
    ::on_shutdown();
    0
}
#[no_mangle]
pub extern "stdcall" fn native_enable() -> i32 {
    unsafe {
        if let Some(dispatcher) = DISPATCHER.as_mut() {
            dispatcher.enable();
        }
    }
    0
}
#[no_mangle]
pub extern "stdcall" fn native_disable() -> i32 {
    unsafe {
        if let Some(dispatcher) = DISPATCHER.as_mut() {
            dispatcher.disable();
        }
    }
    0
}
#[no_mangle]
pub extern "stdcall" fn native_on_recv_priv(subtype: i32,
                                            msgId: i32,
                                            from_qq: i64,
                                            msg: *const c_char,
                                            font: i32) -> i32 {
    let raw = unsafe { CStr::from_ptr(msg) };
    use encoding_rs::GB18030;
    let (decoded, _) = GB18030.decode_without_bom_handling(raw.to_bytes());
    unsafe {
        if let Some(dispatcher) = DISPATCHER.as_ref() {
            let msg = dispatcher.composer().decompose(&decoded).unwrap();
            let msg_in = make_priv_msg_in(from_qq, msg);
            dispatcher.dispatch(msg_in);
        }
    }
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
    let (decoded, _) = GB18030.decode_without_bom_handling(raw.to_bytes());
    unsafe {
        if let Some(dispatcher) = DISPATCHER.as_ref() {
            let msg = dispatcher.composer().decompose(&decoded).unwrap();
            let msg_in = make_grp_msg_in(from_grp, from_qq, msg);
            dispatcher.dispatch(msg_in);
        }
    }
    consts::EVENT_IGNORE
}
