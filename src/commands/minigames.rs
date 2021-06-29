use rand::random;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use std::time::Duration;
#[command]
pub async fn challenge(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.mentions.is_empty() {
        msg.reply(ctx, "mention to daal chutiye").await?;
        return Ok(());
    }
    let challenge = msg
        .channel_id
        .say(
            ctx,
            format!(
                "{}, reply to this message to accept the challenge by {} in 60s [y/n]",
                msg.mentions[0].mention(),
                msg.author.mention()
            ),
        )
        .await?;
    if let Some(answer) = &msg.mentions[0]
        .await_reply(&ctx)
        .timeout(Duration::from_secs(60))
        .await
    {
        if ["yes", "y", "ha"].contains(&answer.content.to_lowercase().as_str()) {
            answer
                .reply(ctx, format!("Challenge accepted, {}", msg.author.mention()))
                .await?;
            challenge.delete(&ctx.http).await?;
        } else if ["no", "nahi", "n"].contains(&answer.content.to_lowercase().as_str()) {
            msg.reply(ctx, "Challenge not accepted").await?;
            challenge.delete(&ctx.http).await?;
            return Ok(());
        } else {
            answer
                .reply(
                    ctx,
                    "Please only answer in no/nahi/n or yes/ha/y\ndeleting challenge ...",
                )
                .await?;
        }
    } else {
        msg.reply(ctx, "Challenge not accepted in 60s").await?;
        return Ok(());
    }
    let won = random();
    let winner = if won {
        msg.mentions[0].mention()
    } else {
        msg.author.mention()
    };
    let loser = if won {
        msg.author.mention()
    } else {
        msg.mentions[0].mention()
    };
    msg.reply(ctx, format!("{} won, {} ki ma ki chut", winner, loser))
        .await?;
    Ok(())
}
