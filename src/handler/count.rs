use regex::Regex;
use serenity::model::channel::Message;
use tokio_postgres::Client;

pub async fn count(msg: Message, db: std::sync::Arc<Client>) {
    let id = msg.author.id.as_u64().to_owned().to_string();
    db.execute(
        format!(
            r#"
            CREATE TABLE IF NOT EXISTS user{} (
            id              SERIAL PRIMARY KEY,
            name            VARCHAR NOT NULL,
            count           INTEGER NOT NULL
            )"#,
            id
        )
        .as_str(),
        &[],
    )
    .await
    .expect("cant create a user table");

    for row in db
        .query("SELECT name, reg FROM words", &[])
        .await
        .expect("can't get the words to count")
    {
        let name: &str = row.get(0);
        let regex: Regex = Regex::new(row.get(1)).unwrap();
        let count = regex.captures_iter(&msg.content).count();
        if count > 0 {
            let query_result = db
                .query(
                    format!("SELECT count FROM user{} where name='{}'", id, name).as_str(),
                    &[],
                )
                .await
                .expect("cant select the count");
            if query_result.is_empty() {
                db.execute(
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
            db.execute(
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
