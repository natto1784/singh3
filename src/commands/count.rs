use crate::lib::components::make_terminal_components;
use core::time::Duration;
use regex::Regex;
use serenity::{
    builder::CreateEmbed,
    collector::component_interaction_collector::ComponentInteractionCollectorBuilder,
    framework::standard::{macros::command, Args, CommandResult},
    futures::StreamExt,
    model::{interactions::InteractionResponseType, prelude::*},
    prelude::*,
    utils::Colour,
};
use tokio_postgres::Row;

#[command]
#[aliases("kitna", "c")]
pub async fn count(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query: String = args.raw().collect::<Vec<&str>>().join(" ");
    if query == "" {
        msg.reply(ctx, "Count what?").await?;
        return Ok(());
    }
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();

    let id = msg.author.id.to_string();
    let mut query_helper = db
        .query("SELECT name FROM words WHERE $1 ~ reg", &[&query])
        .await?;

    if query_helper.is_empty() {
        query_helper = db
            .query("SELECT name FROM words WHERE name=$1", &[&query])
            .await?;
        if query_helper.is_empty() {
            msg.reply(
                ctx,
                format!(
                    "No entry for '{}' found. If you want to add it, run `,cadd {}&<regex>`",
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
        let count_query = db
            .query(
                format!("SELECT count FROM user{} WHERE name=$1", id).as_str(),
                &[&name],
            )
            .await?;
        let query_result = if count_query.is_empty() {
            0
        } else {
            count_query[0].get(0)
        };
        reply += &format!("\n{} count for you: {}", name, query_result);
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
    let r = Regex::new(&format!("(?i){}", queries[1]));

    if r.is_err() {
        msg.reply(ctx, "Please enter a valid regex").await?;
        return Ok(());
    }

    let reg = r.unwrap();

    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();

    let check_existense = db
        .query("SELECT name, reg FROM words WHERE name=$1", &[&queries[0]])
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
        "INSERT INTO words(name, reg, owner) VALUES($1, $2, $3)",
        &[&queries[0], &reg.to_string(), &msg.author.id.to_string()],
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
        .query("SELECT owner FROM words WHERE name=$1", &[&query])
        .await?;
    if owner.len() == 1 {
        let owner_id: String = owner[0].get(0);
        if owner_id != msg.author.id.to_string() {
            msg.reply(ctx, "You don't even own this word").await?;
            return Ok(());
        }
    }
    db.execute("DELETE FROM words WHERE name=$1", &[&query])
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
        .query("SELECT owner FROM words WHERE name=$1", &[&queries[0]])
        .await?;
    if owner.len() == 1 {
        let owner_id: String = owner[0].get(0);
        if owner_id != msg.author.id.to_string() {
            msg.reply(ctx, "You don't even own this word").await?;
            return Ok(());
        }
    }
    db.execute(
        "UPDATE words SET reg=$1 WHERE name=$2",
        &[&("(?i)".to_string() + queries[1]), &queries[0]],
    )
    .await?;
    msg.reply(ctx, "Changed the value if it existed").await?;
    Ok(())
}

fn make_list_embed(cur: usize, group: &[Row]) -> CreateEmbed {
    let mut e = CreateEmbed::default();
    e.title(format!("List of words: Page {}", cur))
        .color(Colour::TEAL);
    for row in group {
        let idx: i64 = row.get(0);
        let name: String = row.get(1);
        let owner_id: String = row.get(2);
        e.field(
            format!("{}. {}", idx, name),
            format!(" by <@{}>", owner_id),
            false,
        );
    }
    e
}

#[command]
#[aliases("cls")]
pub async fn clist(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let size = if args.len() > 0 {
        args.single::<usize>()?
    } else {
        5usize
    };

    if size > 15 {
        msg.reply(ctx, "Please input a number less than 15").await?;
        ()
    }

    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();
    let rows = db
        .query(
            "SELECT ROW_NUMBER() OVER (ORDER BY id), name, owner FROM words",
            &[],
        )
        .await?;
    if rows.is_empty() {
        msg.reply(ctx, "No words stored").await?;
        return Ok(());
    }
    let groups: Vec<&[Row]> = rows.chunks(size).collect();
    let mut cur = 1;

    let message = msg
        .channel_id
        .send_message(ctx, |m| {
            m.set_embed(make_list_embed(cur, groups[cur - 1]))
                .set_components(make_terminal_components("first", groups.len()))
        })
        .await?;

    let mut collector = ComponentInteractionCollectorBuilder::new(&ctx)
        .timeout(Duration::from_secs(90))
        .author_id(msg.author.id)
        .message_id(message.id)
        .await;

    while let Some(interaction) = collector.next().await {
        match interaction.data.custom_id.as_ref() {
            "next" => {
                if cur != groups.len() {
                    cur += 1;
                    let _ = interaction
                        .create_interaction_response(&ctx, |r| {
                            r.kind(InteractionResponseType::UpdateMessage)
                                .interaction_response_data(|m| {
                                    m.add_embed(make_list_embed(cur, groups[cur - 1]))
                                        .set_components(make_terminal_components(
                                            if cur == groups.len() { "last" } else { "mid" },
                                            groups.len(),
                                        ))
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
                                    m.add_embed(make_list_embed(cur, groups[cur - 1]))
                                        .set_components(make_terminal_components(
                                            if cur == 1 { "first" } else { "mid" },
                                            groups.len(),
                                        ))
                                })
                        })
                        .await;
                }
            }
            "first" => {
                cur = 1;
                let _ = interaction
                    .create_interaction_response(&ctx, |r| {
                        r.kind(InteractionResponseType::UpdateMessage)
                            .interaction_response_data(|m| {
                                m.add_embed(make_list_embed(cur, groups[cur - 1]))
                                    .set_components(make_terminal_components("first", groups.len()))
                            })
                    })
                    .await;
            }
            "last" => {
                cur = groups.len();
                let _ = interaction
                    .create_interaction_response(&ctx, |r| {
                        r.kind(InteractionResponseType::UpdateMessage)
                            .interaction_response_data(|m| {
                                m.add_embed(make_list_embed(cur, groups[cur - 1]))
                                    .set_components(make_terminal_components("last", groups.len()))
                            })
                    })
                    .await;
            }
            "delete" => {
                message.delete(ctx).await?;
                msg.delete(ctx).await?;
            }
            "range" => {
                cur = interaction.data.values[0].parse().unwrap();
                let _ = interaction
                    .create_interaction_response(&ctx, |r| {
                        r.kind(InteractionResponseType::UpdateMessage)
                            .interaction_response_data(|m| {
                                m.add_embed(make_list_embed(cur, groups[cur - 1]))
                                    .set_components(make_terminal_components(
                                        if cur == 1 {
                                            "first"
                                        } else if cur == groups.len() {
                                            "last"
                                        } else {
                                            "mid"
                                        },
                                        groups.len(),
                                    ))
                            })
                    })
                    .await;
            }
            _ => {}
        }
    }
    Ok(())
}
