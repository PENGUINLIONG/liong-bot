//! Import symbols from `CQP.dll`
use std::cell::Cell;
use std::ffi::CStr;
use std::os::raw::c_char;
use ::Dispatcher;

mod consts {
    pub const APP_INFO: &'static str = "9,moe.penguinliong.liongbot";

    pub const EVENT_IGNORE: i32 = 0;
    pub const EVENT_BLOCK: i32 = 1;

    pub const REQUEST_ALLOW: i32 = 1;
    pub const REQUEST_DENY: i32 = 2;

    pub const REQUEST_GROUP_ADD: i32 = 1;
    pub const REQUEST_GROUP_INVITE: i32 = 2;
}

static mut AUTH: i32 = 0;
static mut DISPATCHER: Option<Dispatcher> = None;

#[no_mangle]
#[allow(non_snake_case)]
extern {
    /// Send private message. Message ID is returned on success.
    pub fn CQ_sendPrivateMsg(auth: i32, qq: i64, msg: *const c_char) -> i32;

    /// Send group message. Message ID is returned on success.
    pub fn CQ_sendGroupMsg(auth: i32, grp: i64, msg: *const c_char) -> i32;

    /// Get group member user info.
    pub fn CQ_getGroupMemberInfoV2(auth: i32,
                                   qq: i64,
                                   no_cache: i32) -> *const c_char;

    // CoolQ will call the following functions.
}
pub extern fn AppInfo() -> *const u8 {
    consts::APP_INFO.as_ptr()
}
pub extern fn Initialize(auth: i32) -> i32 {
    AUTH = auth;
    0
}
pub extern fn __eventStartup() -> i32 {
    let d = Some(Dispatcher::new());
    ::configure(d);
    DISPATCHER = d;
    0
}
pub extern fn __eventExit() -> i32 {
    0
}
pub extern fn __eventEnable() -> i32 {
    0
}
pub extern fn __eventDisable() -> i32 {
    0
}
pub extern fn __eventPrivateMsg(subtype: i32,
                            msgId: i32,
                            from_qq: i64,
                            msg: *const char,
                            font: i32) -> i32 {
    0
}
pub extern fn __eventGroupMsg(subtype: i32,
                        msgId: i32,
                        from_grp: i64,
                        from_qq: i64,
                        from_anon: *const char,
                        msg: *const char,
                        font: i32) -> i32 {
    0
}
pub extern fn __eventDiscussMsg(subtype: i32,
                            msgId: i32,
                            from_discuss: i64,
                            from_qq: i64,
                            msg: *const char,
                            font: i32) -> i32 {
    0
}
pub extern fn __eventSystem_GroupAdmin(subtype: i32,
                                send_time: i32,
                                from_grp: i64,
                                target_qq: i64) -> i32 {
    0
}
pub extern fn __eventSystem_GroupMemberDecrease(subtype: i32,
                                            send_time: i32,
                                            from_grp: i64,
                                            target_qq: i64) -> i32 {
    0
}
pub extern fn __eventSystem_GroupMemberIncrease(subtype: i32,
                                            send_time: i32,
                                            from_grp: i64,
                                            target_qq: i64) -> i32 {
    0
}
pub extern fn __eventFriend_Add(subtype: i32,
                            send_time: i32,
                            target_qq: i64) -> i32 {
    0
}
pub extern fn __eventRequest_AddFriend(subtype: i32,
                                send_time: i32,
                                from_qq: i64,
                                msg: *const char,
                                res_flag: *const char) -> i32 {
    0
}
pub extern fn __eventRequest_AddGroup(subtype: i32,
                                send_time: i32,
                                from_grp: i64,
                                from_qq: i64,
                                msg: *const char,
                                res_flag: *const char) -> i32 {
    0
}
