use crate::lib::{components::make_terminal_components, messages::ExtractInfo};
use core::time::Duration;
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

const GUILD_ID: u64 = 874699899067838535;
const ROLE_ID: u64 = 957155053184102400;

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
        const DIST_LIMIT: i32 = 2;
        let leven = db
            .query(
                "SELECT name, levenshtein_less_equal(name, $1, $2) AS lev FROM tags ORDER BY lev LIMIT 1 ",
                &[&query, &DIST_LIMIT],
            )
            .await?;

        let dist: i32 = leven[0].get(1);

        let l = if dist > DIST_LIMIT {
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
pub async fn tadd(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let tag_value = msg.extract_text(2, true);

    if tag_value.is_none() {
        msg.reply(
            ctx,
            "Please use the proper syntax: `,tadd <name> <value>` or attach something",
        )
        .await?;
        return Ok(());
    }

    let tag_name = args.single::<String>().unwrap();
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();
    let check_existense = db
        .query("SELECT name FROM tags WHERE name=$1", &[&tag_name])
        .await?;
    if check_existense.len() != 0 {
        msg.reply(ctx, format!("This tag already exists")).await?;
        return Ok(());
    }
    db.execute(
        "INSERT INTO tags(name, value, owner) VALUES($1, $2, $3)",
        &[&tag_name, &tag_value, &msg.author.id.to_string()],
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
    msg.reply(ctx, format!("Copied {} to {}", queries[0], queries[1]))
        .await?;
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
        if owner_id != msg.author.id.to_string()
            && !msg.author.has_role(&ctx.http, GUILD_ID, ROLE_ID).await?
        {
            msg.reply(ctx, "You don't even own this tag").await?;
            return Ok(());
        }
    }
    db.execute("DELETE FROM tags WHERE name=$1", &[&query])
        .await?;
    msg.reply(ctx, format!("Deleted {} if it existed", query))
        .await?;
    Ok(())
}

#[command]
pub async fn tedit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let tag_value = msg.extract_text(2, true);

    if tag_value.is_none() {
        msg.reply(
            ctx,
            "Please use the proper syntax: `,tadd <name> <value>` or attach something",
        )
        .await?;
        return Ok(());
    }

    let tag_name = args.single::<String>().unwrap();
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<crate::Database>()
        .expect("Expected Database in TypeMap.")
        .clone();
    let owner = db
        .query("SELECT owner FROM tags WHERE name=$1", &[&tag_name])
        .await?;
    if owner.len() == 1 {
        let owner_id: String = owner[0].get(0);
        if owner_id != msg.author.id.to_string()
            && !msg.author.has_role(&ctx.http, GUILD_ID, ROLE_ID).await?
        {
            msg.reply(ctx, "You don't even own this tag").await?;
            return Ok(());
        }
    }
    db.execute(
        "UPDATE tags SET value=$1 WHERE name=$2",
        &[&tag_value, &tag_name],
    )
    .await?;
    msg.reply(ctx, "Changed the value if it existed").await?;
    Ok(())
}

fn make_list_embed(cur: usize, group: &[Row]) -> CreateEmbed {
    let mut e = CreateEmbed::default();
    e.title(format!("List of tags: Page {}", cur))
        .color(Colour::FABLED_PINK);
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
