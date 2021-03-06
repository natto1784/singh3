mod commands;
mod handler;
mod lib;
use commands::count::*;
use commands::general::*;
use commands::minigames::*;
use commands::tags::*;
use handler::Handler;
use serenity::{
    client::bridge::gateway::ShardManager,
    framework::{
        standard::{
            help_commands,
            macros::{group, help},
            Args, CommandGroup, CommandResult, HelpOptions,
        },
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, id::UserId},
    prelude::*,
};
use std::{collections::HashSet, env, sync::Arc};
use tracing::error;

struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Database;
impl TypeMapKey for Database {
    type Value = Arc<tokio_postgres::Client>;
}

#[group]
#[commands(ping)]
struct General;

#[group]
#[commands(count, cadd, cremove, cedit, clist)]
struct Count;

#[group]
#[commands(tag, tadd, tcopy, tremove, tedit, tlist, trandom)]
pub struct Tags;

#[group]
#[commands(challenge)]
struct Minigames;

#[help]
#[max_levenshtein_distance(2)]
#[indention_prefix = "+"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Token daal madarchod");
    let http = Http::new_with_token(&token);
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix(","))
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&COUNT_GROUP)
        .group(&TAGS_GROUP)
        .group(&MINIGAMES_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .application_id(*bot_id.as_u64())
        .await
        .expect("Client no wokey");

    {
        let db_url: String = env::var("DB_URL").expect("DB_URL not found");
        let (db_client, conn) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
            .await
            .expect("cant connect bha");
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("connection error: {}", e);
            }
        });
        let init_script = std::include_str!("../init.sql");
        db_client
            .batch_execute(init_script)
            .await
            .expect("Couldn't run the init script");
        let mut data = client.data.write().await;
        data.insert::<Database>(Arc::new(db_client));
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
