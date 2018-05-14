use super::super::VoiceManager;
use super::super::check_msg;
use super::super::get_guild_id;

use reqwest;
use serenity::voice;
use std::env;
use std::io::BufWriter;
use std::io::Write;
use std::io::copy;
use std::io::stdout;
use tempfile::NamedTempFile;

use typemap::Key;

pub struct VolumeParameter;

impl Key for VolumeParameter {
    type Value = f32;
}

command!(tts(ctx, msg, args) {
    let data = ctx.data.lock();
    let vol = data.get::<VolumeParameter>().unwrap_or(&1.0);

    let key = env::var("VOICE_RSS_KEY").expect("Expected a key for voice-rss in conf.env");
    let src: String = args.multiple::<String>().unwrap().join(" ");
    let mut response = reqwest::get(&format!("https://api.voicerss.org/?key={}&hl={}&src={}", key, "de-de", src))
    .expect("Failed to send request");

    println!("===== Response for {} =====", src);
    println!("{}", response.status());
    for header in response.headers().iter() {
        println!("{}: {}", header.name(), header.value_string());
    }
    println!("===== Response end =====");
    
    let mut tmpfile = NamedTempFile::new()?;
    let mut writer = BufWriter::new(&tmpfile);
    copy(&mut response, &mut writer).expect("Failed to save response to tempfile");
    writer.flush().unwrap();
    println!("Saved temporary tts-file to {:?}", &tmpfile.path());
    println!("Content of temporary file:");
    let mut stdout = stdout();
    copy(&mut response, &mut stdout).expect("Failed to copy response to stdout");

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
        let audio = handler.play_only(source);
        let mut lock = audio.lock();
        lock.volume(*vol);
    } else {
        check_msg(msg.channel_id.say("Not in a voice channel to play in"));
    }
});

command!(volume(ctx, msg, args) {
    let vol = match args.single::<f32>() {
        Ok(vol) if vol >= 0.0 && vol <= 1.0 => vol,
        _ => {
            check_msg(msg.channel_id.say("Volume must be between 0.0 and 1.0"));
            return Ok(());
        }
    };
    let mut data = ctx.data.lock();
    data.insert::<VolumeParameter>(vol);
});

command!(yt(ctx, msg, args) {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(msg.channel_id.say("Must provide a URL to a video or audio"));
            return Ok(());
        },
    };

    let data = ctx.data.lock();
    let vol = data.get::<VolumeParameter>().unwrap_or(&1.0);

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

        let audio = handler.play_only(source);
        let mut lock = audio.lock();
        lock.volume(*vol);
    } else {
        check_msg(msg.channel_id.say("Not in a voice channel to play in"));
    }
});
