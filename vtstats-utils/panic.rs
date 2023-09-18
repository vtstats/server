use backtrace::Backtrace;
use serde::Serialize;
use std::{
    io::{stderr, Write},
    panic::PanicInfo,
};

pub fn hook_impl(info: &PanicInfo) {
    let mut msg = info.to_string();

    if let Some(location) = info.location() {
        msg += &format!(" caller={}:{}", location.file(), location.line());
    }

    let backtrace = Backtrace::new();

    #[derive(Serialize)]
    struct PanicMessage<'a> {
        level: &'a str,
        message: &'a str,
        stack: &'a str,
    }

    let mut stderr = stderr();

    let _ = serde_json::to_writer(
        &stderr,
        &PanicMessage {
            level: "FATAL",
            message: &msg,
            stack: &format!("{:?}", backtrace),
        },
    );

    let _ = stderr.write_all(b"\n");
}
