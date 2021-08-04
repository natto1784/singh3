use regex::Regex;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use std::{collections::HashMap, env};
use tokio_postgres::NoTls;

#[command]
pub async fn kitna(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query: String = args.raw().collect::<Vec<&str>>().join(" ");
    if query == "" {
        msg.reply(ctx, "bruh kitna kya?");
    }
    let words: HashMap<&str, Regex> = [
        ("nword", Regex::new(r"(?i)(nig+(er|a)|nig{2,})").unwrap()),
        ("acha", Regex::new(r"(?i)a((c+?h+?|6+?)a+").unwrap()),
        ("sus", Regex::new(r"(?i)sus|(?i)amon??g\s??us").unwrap()),
    ]
    .iter()
    .cloned()
    .collect();
    let db: String = env::var("DB_URL").expect("bhay DB_URL daal na");
    let (client, conn) = tokio_postgres::connect(&db, NoTls)
        .await
        .expect("cant connect bha");
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });
    let id = msg.author.id.as_u64().to_owned().to_string();
    for (name, regex) in words {
        if regex.is_match(&query) || &query == name {
            let query_result = client
                .query_one(
                    format!("SELECT count FROM user{} where name='{}'", id, name).as_str(),
                    &[],
                )
                .await
                .expect("cant select the count");
            let count: i32 = query_result.get("count");
            msg.reply(ctx, format!("{} count for you: {}", name, count))
                .await?;
            break;
        }
    }
    Ok(())
}
