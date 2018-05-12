use super::super::check_msg;

use std::sync::atomic::{AtomicBool, Ordering};

static SHOULD_RUN: AtomicBool = AtomicBool::new(true);

pub fn run() {
    if SHOULD_RUN.load(Ordering::Relaxed) {
        println!("Giantbomb task runs now");
    }
}

command!(gbstart(_ctx, msg, _args) {
    SHOULD_RUN.store(true, Ordering::Relaxed);
    check_msg(msg.channel_id.say("Giantbomb task is resumed"));
});

command!(gbpause(_ctx, msg, _arg) {
    SHOULD_RUN.store(false, Ordering::Relaxed);
    check_msg(msg.channel_id.say("Giantbomb task is paused"));
});
