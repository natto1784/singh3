use core::time::Duration;
use serenity::{
    collector::component_interaction_collector::ComponentInteractionCollectorBuilder,
    framework::standard::{macros::command, Args, CommandResult},
    futures::StreamExt,
    model::{
        channel::ReactionType,
        interactions::{ButtonStyle, InteractionData},
        prelude::*,
    },
    prelude::*,
    utils::Colour,
};
use tokio_postgres::Row;

#[command]
#[aliases("kitna", "c")]
pub async fn count(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query: String = args.raw().collect::<Vec<&str>>().join(" ");
    if query == "" {
        msg.reply(ctx, "bruh kitna kya?").await?;
        return Ok(());
    }
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();

    let id = msg.author.id.to_string();
    let mut query_helper = db
        .query(
            format!("SELECT name FROM words WHERE '{}' ~ reg", query).as_str(),
            &[],
        )
        .await?;
    if query_helper.is_empty() {
        query_helper = db
            .query(
                format!("SELECT name FROM words WHERE name='{}'", query).as_str(),
                &[],
            )
            .await?;
        if query_helper.is_empty() {
            msg.reply(
                ctx,
                format!(
                    "No entry for '{}' found. If you want to add it, run ',cadd {}&<regex>'",
                    query, query
                ),
            )
            .await?;
            return Ok(());
        }
    }
    let mut reply: String = if query_helper.len() == 1 {
        String::new()
    } else {
        format!("{} patterns matched", query_helper.len())
    };
    for row in query_helper {
        let name: &str = row.get(0);
        let query_result: i32 = db
            .query_one(
                format!("SELECT count FROM user{} WHERE name='{}'", id, name).as_str(),
                &[],
            )
            .await?
            .get(0);
        reply = reply + &format!("\n{} count for you: {}", name, query_result);
    }
    msg.reply(ctx, reply).await?;
    Ok(())
}

#[command]
pub async fn cadd(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query: String = args.raw().collect::<Vec<&str>>().join(" ");
    let queries = query.splitn(2, "&").collect::<Vec<&str>>();
    if queries.len() != 2 {
        msg.reply(ctx, "Please use the proper syntax: `,cadd <name>&<regex>`\nIf you don't know what regex is, just do: `,cadd <name>&<name>`")
            .await?;
        return Ok(());
    }
    if queries[1].contains(" ") {
        msg.reply(ctx, "Not a valid regex").await?;
        return Ok(());
    }
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();
    let check_existense = db
        .query(
            format!("SELECT name, reg FROM words WHERE name='{}'", queries[0]).as_str(),
            &[],
        )
        .await?;
    if check_existense.len() != 0 {
        let reg: String = check_existense[0].get(1);
        msg.reply(
            ctx,
            format!("This word already exists with the regex '{}'", reg),
        )
        .await?;
        return Ok(());
    }
    db.execute(
        format!(
            "INSERT INTO words(name, reg, owner) VALUES('{}','(?i){}', '{}')",
            queries[0],
            queries[1],
            msg.author.id.to_string()
        )
        .as_str(),
        &[],
    )
    .await?;
    msg.reply(ctx, "Added").await?;
    Ok(())
}

#[command]
#[aliases("crm")]
pub async fn cremove(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query: String = args.raw().collect::<Vec<&str>>().join(" ");
    if query == "" {
        msg.reply(ctx, "remove what?").await?;
        return Ok(());
    }
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();
    let owner = db
        .query(
            format!("SELECT owner FROM words WHERE name = '{}'", query).as_str(),
            &[],
        )
        .await?;
    if owner.len() == 1 {
        let owner_id: String = owner[0].get(0);
        if owner_id != msg.author.id.to_string() {
            msg.reply(ctx, "You don't even own this word").await?;
            return Ok(());
        }
    }
    db.execute(
        format!("DELETE FROM words WHERE name='{}'", query,).as_str(),
        &[],
    )
    .await?;
    msg.reply(ctx, "Deleted if it existed").await?;
    Ok(())
}

#[command]
pub async fn cedit(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query: String = args.raw().collect::<Vec<&str>>().join(" ");
    let queries = query.splitn(2, "&").collect::<Vec<&str>>();
    if queries.len() != 2 {
        msg.reply(ctx, "Please use the proper syntax\n,cedit <name>&<regex>")
            .await?;
        return Ok(());
    }
    if queries[1].contains(" ") {
        msg.reply(ctx, "Not a valid regex").await?;
        return Ok(());
    }
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();
    let owner = db
        .query(
            format!("SELECT owner FROM words WHERE name = '{}'", queries[0]).as_str(),
            &[],
        )
        .await?;
    if owner.len() == 1 {
        let owner_id: String = owner[0].get(0);
        if owner_id != msg.author.id.to_string() {
            msg.reply(ctx, "You don't even own this word").await?;
            return Ok(());
        }
    }
    db.execute(
        format!(
            "UPDATE words SET reg='(?i){}' WHERE name='{}'",
            queries[1], queries[0]
        )
        .as_str(),
        &[],
    )
    .await?;
    msg.reply(ctx, "Changed the value if it existed").await?;
    Ok(())
}

macro_rules! make_embed {
    ($e: expr, $cur: expr, $group: expr) => {{
        $e = $e
            .title(format!("List of words: Page {}", $cur))
            .color(Colour::TEAL);
        for row in $group {
            let idx: i64 = row.get(0);
            let name: String = row.get(1);
            let owner_id: String = row.get(2);
            $e = $e.field(
                format!("{}. {}", idx, name),
                format!(" by <@{}>", owner_id),
                true,
            );
        }
        $e
    }};
}

macro_rules! make_terminal_components {
    ($c: expr, $terminal: expr ) => {{
        $c.create_action_row(|ar| {
            ar.create_button(|b| {
                b.style(ButtonStyle::Primary)
                    .label("Prev")
                    .emoji(ReactionType::Unicode("\u{2B05}".to_string()))
                    .custom_id("prev")
                    .disabled($terminal == "first")
            })
            .create_button(|b| {
                b.style(ButtonStyle::Primary)
                    .label("Next")
                    .emoji(ReactionType::Unicode("\u{27A1}".to_string()))
                    .custom_id("next")
                    .disabled($terminal == "last")
            })
            .create_button(|b| {
                b.style(ButtonStyle::Danger)
                    .label("Delete")
                    .emoji(ReactionType::Unicode("\u{1F5D1}".to_string()))
                    .custom_id("delete")
            })
        })
    }};
}

#[command]
#[aliases("clist")]
pub async fn clist(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();
    let rows = db
        .query("SELECT ROW_NUMBER() OVER (ORDER BY id), name, owner FROM words", &[])
        .await?;
    if rows.is_empty() {
        msg.reply(ctx, "No words stored").await?;
        return Ok(());
    }
    let groups: Vec<&[Row]> = rows.chunks(5).collect();
    let mut cur = 1;

    let message = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|mut e| make_embed!(e, cur, groups[cur - 1]))
                .components(|c| make_terminal_components!(c, "first"))
        })
        .await?;
    let mut collector = ComponentInteractionCollectorBuilder::new(&ctx)
        .timeout(Duration::from_secs(90))
        .author_id(msg.author.id)
        .message_id(message.id)
        .await;
    while let Some(interaction) = collector.next().await {
        if let InteractionData::MessageComponent(component) = interaction.data.as_ref().unwrap() {
            match component.custom_id.as_ref() {
                "next" => {
                    if cur != groups.len() {
                        cur += 1;
                        let _ = interaction
                            .create_interaction_response(&ctx, |r| {
                                r.kind(InteractionResponseType::UpdateMessage)
                                    .interaction_response_data(|m| {
                                        m.create_embed(|mut e| make_embed!(e, cur, groups[cur - 1]))
                                            .components(|c| {
                                                make_terminal_components!(
                                                    c,
                                                    if cur == groups.len() {
                                                        "last"
                                                    } else {
                                                        "mid"
                                                    }
                                                )
                                            })
                                    })
                            })
                            .await;
                    }
                }
                "prev" => {
                    if cur != 1 {
                        cur -= 1;
                        let _ = interaction
                            .create_interaction_response(&ctx, |r| {
                                r.kind(InteractionResponseType::UpdateMessage)
                                    .interaction_response_data(|m| {
                                        m.create_embed(|mut e| make_embed!(e, cur, groups[cur - 1]))
                                            .components(|c| {
                                                make_terminal_components!(
                                                    c,
                                                    if cur == 1 { "first" } else { "mid" }
                                                )
                                            })
                                    })
                            })
                            .await;
                    }
                }
                "delete" => {
                    message.delete(ctx).await?;
                    msg.delete(ctx).await?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
