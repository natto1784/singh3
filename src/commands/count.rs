use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Colour,
};
use std::env;
use tokio_postgres::NoTls;

#[command]
pub async fn kitna(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query: String = args.raw().collect::<Vec<&str>>().join(" ");
    if query == "" {
        msg.reply(ctx, "bruh kitna kya?").await?;
        return Ok(());
    }
    let db: String = env::var("DB_URL").expect("bhay DB_URL daal na");
    let (client, conn) = tokio_postgres::connect(&db, NoTls)
        .await
        .expect("cant connect bha");
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });
    let id = msg.author.id.to_string();
    let mut query_helper = client
        .query(
            format!("select name from words where '{}' ~ reg", query).as_str(),
            &[],
        )
        .await
        .expect("helper query to select count failed");
    if query_helper.is_empty() {
        query_helper = client
            .query(
                format!("select name from words where name='{}'", query).as_str(),
                &[],
            )
            .await
            .expect("helper query to select count failed");
        if query_helper.is_empty() {
            msg.reply(
                ctx,
                format!(
                    "No entry for '{}' found. If you want to add it, run 'xxadd {}&<regex>'",
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
        let query_result: i32 = client
            .query_one(
                format!("select count from user{} where name='{}'", id, name).as_str(),
                &[],
            )
            .await
            .expect("cant select the count")
            .get(0);
        reply = reply + &format!("\n{} count for you: {}", name, query_result);
    }
    msg.reply(ctx, reply).await?;
    Ok(())
}

#[command]
pub async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query: String = args.raw().collect::<Vec<&str>>().join(" ");
    let queries = query.split("&").collect::<Vec<&str>>();
    if queries.len() != 2 {
        msg.reply(ctx, "Please use the proper syntax: xxadd <name>&<regex>\nIf you don't know what regex is, just do: xxadd <name>&<name>")
            .await?;
        return Ok(());
    }
    if queries[1].contains(" ") {
        msg.reply(ctx, "Not a valid regex").await?;
        return Ok(());
    }
    let db: String = env::var("DB_URL").expect("bhay DB_URL daal na");
    let (client, conn) = tokio_postgres::connect(&db, NoTls)
        .await
        .expect("cant connect bha");
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });
    let check_existense = client
        .query(
            format!("select name, reg from words where name='{}'", queries[0]).as_str(),
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
    client
        .execute(
            format!(
                "insert into words(name, reg, owner) values('{}','(?i){}', '{}')",
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
pub async fn rm(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query: String = args.raw().collect::<Vec<&str>>().join(" ");
    let db: String = env::var("DB_URL").expect("bhay DB_URL daal na");
    let (client, conn) = tokio_postgres::connect(&db, NoTls)
        .await
        .expect("cant connect bha");
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });
    let owner = client
        .query(
            format!("select owner from words where name = '{}'", query).as_str(),
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
    client
        .execute(
            format!("delete from words where name='{}'", query,).as_str(),
            &[],
        )
        .await?;
    msg.reply(ctx, "Deleted if it existed").await?;
    Ok(())
}

#[command]
pub async fn change(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query: String = args.raw().collect::<Vec<&str>>().join(" ");
    let queries = query.split("&").collect::<Vec<&str>>();
    if queries.len() != 2 {
        msg.reply(ctx, "Please use the proper syntax\nxxchange <name>&<regex>")
            .await?;
        return Ok(());
    }
    if queries[1].contains(" ") {
        msg.reply(ctx, "Not a valid regex").await?;
        return Ok(());
    }
    let db: String = env::var("DB_URL").expect("bhay DB_URL daal na");
    let (client, conn) = tokio_postgres::connect(&db, NoTls)
        .await
        .expect("cant connect bha");
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });
    let owner = client
        .query(
            format!("select owner from words where name = '{}'", queries[0]).as_str(),
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
    client
        .execute(
            format!(
                "update words set reg='(?i){}' where name='{}'",
                queries[1], queries[0]
            )
            .as_str(),
            &[],
        )
        .await?;
    msg.reply(ctx, "Changed the value if it existed").await?;
    Ok(())
}

#[command]
pub async fn list(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let db: String = env::var("DB_URL").expect("bhay DB_URL daal na");
    let (client, conn) = tokio_postgres::connect(&db, NoTls)
        .await
        .expect("cant connect bha");
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });
    let rows = client.query("select * from words", &[]).await?;
    msg.channel_id
        .send_message(ctx, |mut m| {
            let mut a: u32 = 1;
            for group in rows.chunks(5) {
                m = m.embed(|mut e| {
                    e = e
                        .title(format!("List of words: Page {}", a))
                        .color(Colour::TEAL);
                    a += 1;
                    for row in group {
                        let idx: u32 = row.get(0);
                        let name: String = row.get(1);
                        let _reg: String = row.get(2);
                        let owner_id: String = row.get(3);
                        e = e.field(
                            format!("{}. {}", idx, name),
                            format!(" by <@{}>", owner_id),
                            true,
                        );
                    }
                    e
                })
            }
            m
        })
        .await?;
    Ok(())
}
