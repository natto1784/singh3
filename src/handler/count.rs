use regex::Regex;
use serenity::model::channel::Message;
use std::collections::HashMap;
use std::env;
use tokio_postgres::NoTls;

pub async fn count(msg: Message) {
    let words: HashMap<&str, Regex> = [
        ("nword", Regex::new(r"(?i)(nig+(er|a)|nig{2,})").unwrap()),
        ("acha", Regex::new(r"(?i)a(c+?h+?|6+?)a+").unwrap()),
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
    client
        .execute(
            format!(
                "
        CREATE TABLE IF NOT EXISTS user{} (
            id              SERIAL PRIMARY KEY,
            name            VARCHAR NOT NULL,
            count           INTEGER NOT NULL
            )
    ",
                id
            )
            .as_str(),
            &[],
        )
        .await
        .expect("cant create table");

    for name in ["nword", "acha", "sus"] {
        let count = words[name].captures_iter(&msg.content).count();
        if count > 0 {
            let query_result = client
                .query(
                    format!("SELECT count FROM user{} where name='{}'", id, name).as_str(),
                    &[],
                )
                .await
                .expect("cant select the count");
            if query_result.is_empty() {
                client
                    .execute(
                        format!(
                            "insert into user{} (name, count) values ('{}', 0)",
                            id, name
                        )
                        .as_str(),
                        &[],
                    )
                    .await
                    .expect("cant insert shit");
            }
            client
                .execute(
                    format!(
                        "UPDATE user{} SET count = count + {} where name='{}'",
                        id, count, name
                    )
                    .as_str(),
                    &[],
                )
                .await
                .expect("cant update");
        }
    }
}
