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
    .expect("Can't create a user table");

    for row in db
        .query("SELECT name, reg FROM words", &[])
        .await
        .expect("Can't get the words to count")
    {
        let name: &str = row.get(0);
        let regex: Regex = Regex::new(row.get(1)).unwrap();
        let count: i32 = regex.captures_iter(&msg.content).count() as i32;
        if count > 0 {
            let query_result = db
                .query(
                    format!("SELECT count FROM user{} WHERE name=$1", id).as_str(),
                    &[&name],
                )
                .await
                .expect("Can't select count");
            if query_result.is_empty() {
                db.execute(
                    format!("INSERT INTO user{} (name, count) values ($1, 0)", id).as_str(),
                    &[&name],
                )
                .await
                .expect("Can't insert count");
            }
            db.execute(
                format!("UPDATE user{} SET count = count + $1 WHERE name=$2", id).as_str(),
                &[&count, &name],
            )
            .await
            .expect("Can't update count");
        }
    }
}
