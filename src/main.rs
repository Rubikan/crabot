extern crate kankyo;
extern crate reqwest;
#[macro_use]
extern crate serenity;
extern crate chrono;
extern crate tempfile;
extern crate timer;
extern crate typemap;

mod commands;
mod tasks;

use commands::sound::VolumeParameter;
use serenity::Result as SerenityResult;
use serenity::client::CACHE;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::model::id::GuildId;
use serenity::prelude::Mutex;
use serenity::prelude::*;
use std::env;
use std::fs::File;
use std::sync::Arc;
use timer::Timer;
use typemap::Key;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

struct VoiceManager;

impl Key for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let conf_path = &args[1];
    let mut conf_file = File::open(conf_path).expect("Error reading config file");

    if let Err(why) = kankyo::load_from_reader(&mut conf_file) {
        println!("Error loading config file to environment: {:?}", why)
    }

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in conf.env");
    let mut client = Client::new(&token, Handler).expect("Error creating client");

    {
        let mut data = client.data.lock();
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
        data.insert::<VolumeParameter>(0.8);
    }

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .command("join", |c| c.cmd(commands::channels::join))
            .command("leave", |c| c.cmd(commands::channels::leave))
            .command("volume", |c| c.cmd(commands::sound::volume))
            .command("tts", |c| c.cmd(commands::sound::tts))
            .command("yt", |c| c.cmd(commands::sound::yt))
            .command("gbstart", |c| c.cmd(tasks::giantbomb::gbstart))
            .command("gbpause", |c| c.cmd(tasks::giantbomb::gbpause)),
    );

    let timer = Timer::new();

    let gb_guard = {
        timer.schedule_repeating(chrono::Duration::minutes(5), move || {
            tasks::giantbomb::run();
        })
    };

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
        drop(gb_guard);
    }
}

/// Checks the return value of sent messages and prints errors
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

fn get_guild_id(channel_id: ChannelId) -> GuildId {
    match CACHE.read().guild_channel(channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(
                channel_id
                    .say("Error finding channel info. Groups and DMs not supported for joining."),
            );
            unimplemented!();
        }
    }
}
