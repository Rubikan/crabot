use super::super::check_msg;
use super::super::get_guild_id;
use super::super::VoiceManager;

use reqwest;
use serenity::voice;
use std::env;
use std::io::BufWriter;
use std::io::copy;
use std::io::Write;
use tempfile::NamedTempFile;

command!(tts(ctx, msg, args) {
    let key = env::var("VOICE_RSS_KEY").expect("Expected a key for voice-rss in conf.env");
    let src: String = args.multiple::<String>().unwrap().join(" ");
    let mut response = reqwest::get(&format!("https://api.voicerss.org/?key={}&hl={}&src={}", key, "de-de", src))
    .expect("Failed to send request");

    println!("{}", response.status());
    for header in response.headers().iter() {
        println!("{}: {}", header.name(), header.value_string());
    }
    
    let mut tmpfile = NamedTempFile::new()?;
    let mut writer = BufWriter::new(&tmpfile);
    copy(&mut response, &mut writer).expect("Failed to save response to tempfile");
    writer.flush().unwrap();

    let guild_id = get_guild_id(msg.channel_id);

    let mut manager_lock = ctx.data.lock().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        let source = match voice::ffmpeg(&tmpfile.path()) {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);
                check_msg(msg.channel_id.say("Error sourcing ffmpeg"));
                return Ok(());
            },
        };
        handler.play(source);
    } else {
        check_msg(msg.channel_id.say("Not in a voice channel to play in"));
    }
});

command!(ytdl(ctx, msg, args) {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(msg.channel_id.say("Must provide a URL to a video or audio"));
            return Ok(());
        },
    };

    if !url.starts_with("http") {
        check_msg(msg.channel_id.say("Must provide a valid URL"));
        return Ok(());
    }

    let guild_id = get_guild_id(msg.channel_id);

    let mut manager_lock = ctx.data.lock().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        let source = match voice::ytdl(&url) {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                check_msg(msg.channel_id.say("Error sourcing ffmpeg"));

                return Ok(());
            },
        };
        handler.play(source);
    } else {
        check_msg(msg.channel_id.say("Not in a voice channel to play in"));
    }
});