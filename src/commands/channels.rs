use super::super::VoiceManager;
use super::super::check_msg;
use super::super::get_guild_id;

use serenity::model::id::ChannelId;
use serenity::model::misc::Mentionable;

command!(join(ctx, msg, args) {
    let connect_to = match args.single::<u64>() {
        Ok(id) => ChannelId(id),
        Err(_) => {
            check_msg(msg.reply("Requires a valid voice channel ID be given"));

            return Ok(());
        },
    };

    let guild_id = get_guild_id(msg.channel_id);

    let mut manager_lock = ctx.data.lock().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    if manager.join(guild_id, connect_to).is_some() {
        check_msg(msg.channel_id.say(&format!("Joined {}", connect_to.mention())));
    } else {
        check_msg(msg.channel_id.say("Error joining the channel"));
    }
});

command!(leave(ctx, msg) {
    let guild_id = get_guild_id(msg.channel_id);

    let mut manager_lock = ctx.data.lock().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id);
        check_msg(msg.channel_id.say("Left voice channel"));
    } else {
        check_msg(msg.reply("Not in a voice channel"));
    }
});
