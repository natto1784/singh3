use crate::commands::macros::*;
use core::time::Duration;
use serenity::{
    collector::component_interaction_collector::ComponentInteractionCollectorBuilder,
    framework::standard::{macros::command, Args, CommandResult},
    futures::StreamExt,
    model::{
        channel::ReactionType,
        interactions::{message_component::ButtonStyle, InteractionResponseType},
        prelude::*,
    },
    prelude::*,
    utils::Colour,
};
use tokio_postgres::Row;

#[command]
#[aliases("t")]
pub async fn tag(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query: String = args.raw().collect::<Vec<&str>>().join("");
    if query == "" {
        msg.reply(ctx, "Mention the tag retard").await?;
        return Ok(());
    }
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();

    let query_helper = db
        .query("SELECT name, value FROM tags WHERE name=$1", &[&query])
        .await?;
    if query_helper.is_empty() {
        let leven = db
            .query(
                "SELECT name FROM tags WHERE levenshtein(name, $1) < 2",
                &[&query],
            )
            .await?;
        let l = if leven.is_empty() {
            "".to_string()
        } else {
            let leven_name: String = leven[0].get(0);
            format!("\nDid you mean `{}`?", leven_name)
        };

        msg.reply(
            ctx,
            format!(
                "No entry for '{}' found. If you want to add it, run `,tadd {} <value>`{}",
                query, query, l
            ),
        )
        .await?;
        return Ok(());
    }
    let value: String = query_helper[0].get(1);
    msg.reply(ctx, value).await?;
    Ok(())
}

#[command]
pub async fn tadd(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query: String = args.raw().collect::<Vec<&str>>().join(" ");
    let queries = query.splitn(2, " ").collect::<Vec<&str>>();
    if queries.len() != 2 && msg.attachments.len() == 0 {
        msg.reply(
            ctx,
            "Please use the proper syntax: `,tadd <name> <value>` or attach something",
        )
        .await?;
        return Ok(());
    }
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();
    let check_existense = db
        .query("SELECT name FROM tags WHERE name=$1", &[&queries[0]])
        .await?;
    if check_existense.len() != 0 {
        msg.reply(ctx, format!("This tag already exists")).await?;
        return Ok(());
    }
    db.execute(
        "INSERT INTO tags(name, value, owner) VALUES($1, $2, $3)",
        &[
            &queries[0],
            &format!(
                "{}{}",
                if queries.len() == 2 {
                    format!("{}{}", queries[1], '\n')
                } else {
                    "".to_string()
                },
                msg.attachments
                    .iter()
                    .map(|x| x.url.clone())
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            &msg.author.id.to_string(),
        ],
    )
    .await?;
    msg.reply(ctx, "Added").await?;
    Ok(())
}

#[command]
#[aliases("tcp")]
pub async fn tcopy(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let queries: Vec<&str> = args.raw().collect::<Vec<&str>>();
    if queries.len() != 2 {
        msg.reply(
            ctx,
            "Please use the proper syntax: `,tcopy <original> <new>`",
        )
        .await?;
        return Ok(());
    }
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();
    let check_existense = db
        .query("SELECT name FROM tags WHERE name=$1", &[&queries[0]])
        .await?;
    if check_existense.len() == 0 {
        msg.reply(ctx, format!("This tag does not exist")).await?;
        return Ok(());
    }
    db.execute(
        "INSERT INTO tags(name, value, owner) SELECT $1, value, $2 FROM tags WHERE name=$3",
        &[&queries[1], &msg.author.id.to_string(), &queries[0]],
    )
    .await?;
    msg.reply(ctx, "Copied").await?;
    Ok(())
}

#[command]
#[aliases("trm")]
pub async fn tremove(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
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
        .query("SELECT owner FROM tags WHERE name=$1", &[&query])
        .await?;
    if owner.len() == 1 {
        let owner_id: String = owner[0].get(0);
        if owner_id != msg.author.id.to_string() {
            msg.reply(ctx, "You don't even own this tag").await?;
            return Ok(());
        }
    }
    db.execute("DELETE FROM tags WHERE name=$1", &[&query])
        .await?;
    msg.reply(ctx, "Deleted if it existed").await?;
    Ok(())
}

#[command]
pub async fn tedit(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query: String = args.raw().collect::<Vec<&str>>().join(" ");
    let queries = query.splitn(2, " ").collect::<Vec<&str>>();
    if queries.len() != 2 && msg.attachments.len() == 0 {
        msg.reply(
            ctx,
            "Please use the proper syntax or attach something\n`,tedit <name> <value> `",
        )
        .await?;
        return Ok(());
    }
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();
    let owner = db
        .query("SELECT owner FROM tags WHERE name=$1", &[&queries[0]])
        .await?;
    if owner.len() == 1 {
        let owner_id: String = owner[0].get(0);
        if owner_id != msg.author.id.to_string() {
            msg.reply(ctx, "You don't even own this tag").await?;
            return Ok(());
        }
    }
    db.execute(
        "UPDATE tags SET value=$1 WHERE name=$2",
        &[
            &format!(
                "{}{}",
                if queries.len() == 2 {
                    format!("{}{}", queries[1], '\n')
                } else {
                    "".to_string()
                },
                msg.attachments
                    .iter()
                    .map(|x| x.url.clone())
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            &queries[0],
        ],
    )
    .await?;
    msg.reply(ctx, "Changed the value if it existed").await?;
    Ok(())
}

macro_rules! make_embed {
    ($e: expr, $cur: expr, $group: expr) => {{
        $e = $e
            .title(format!("List of tags: Page {}", $cur))
            .color(Colour::FABLED_PINK);
        for row in $group {
            let idx: i64 = row.get(0);
            let name: String = row.get(1);
            let owner_id: String = row.get(2);
            $e = $e.field(
                format!("{}. {}", idx, name),
                format!(" by <@{}>", owner_id),
                false,
            );
        }
        $e
    }};
}

#[command]
#[aliases("tls")]
pub async fn tlist(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
            "SELECT ROW_NUMBER() OVER (ORDER BY id), name, owner FROM tags",
            &[],
        )
        .await?;
    if rows.is_empty() {
        msg.reply(ctx, "No tags stored").await?;
        return Ok(());
    }
    let groups: Vec<&[Row]> = rows.chunks(size).collect();
    let mut cur = 1;

    let message = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|mut e| make_embed!(e, cur, groups[cur - 1]))
                .components(|c| make_terminal_components!(c, "first", groups.len()))
        })
        .await?;
    let mut collector = ComponentInteractionCollectorBuilder::new(&ctx)
        .timeout(Duration::from_secs(90))
        .author_id(msg.author.id)
        .message_id(message.id)
        .await;

    while let Some(interaction) = collector.next().await {
        let custom_id = interaction.data.custom_id.as_ref();
        match custom_id {
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
                                                if cur == groups.len() { "last" } else { "mid" },
                                                groups.len()
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
                                                if cur == 1 { "first" } else { "mid" },
                                                groups.len()
                                            )
                                        })
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
                                m.create_embed(|mut e| make_embed!(e, cur, groups[cur - 1]))
                                    .components(|c| {
                                        make_terminal_components!(c, "first", groups.len())
                                    })
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
                                m.create_embed(|mut e| make_embed!(e, cur, groups[cur - 1]))
                                    .components(|c| {
                                        make_terminal_components!(c, "last", groups.len())
                                    })
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
                                m.create_embed(|mut e| make_embed!(e, cur, groups[cur - 1]))
                                    .components(|c| {
                                        make_terminal_components!(
                                            c,
                                            if cur == 1 {
                                                "first"
                                            } else if cur == groups.len() {
                                                "last"
                                            } else {
                                                "mid"
                                            },
                                            groups.len()
                                        )
                                    })
                            })
                    })
                    .await;
            }
            _ => {}
        }
    }
    Ok(())
}
