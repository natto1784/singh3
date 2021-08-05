use regex::Regex;
use serenity::model::channel::Message;
use std::env;
use tokio_postgres::NoTls;

pub async fn count(msg: Message) {
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

    for row in client
        .query("SELECT name, reg FROM words", &[])
        .await
        .expect("can't get the words to count")
    {
        let name: &str = row.get(0);
        let regex: Regex = Regex::new(row.get(1)).unwrap();
        let count = regex.captures_iter(&msg.content).count();
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
