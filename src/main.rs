mod general;
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::{
        event::ResumedEvent,
        gateway::Ready,
        interactions::{ApplicationCommand, Interaction, InteractionResponseType, InteractionType, InteractionData},
    },
    prelude::*,
};
use std::{collections::HashSet, env, sync::Arc};
use tracing::{error, info};

pub struct ShardManagerContainer;
use general::*;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if interaction.kind == InteractionType::ApplicationCommand {
            if let Some(InteractionData::ApplicationCommand(data)) = interaction.data.as_ref() {
                let content = match data.name.as_str() {
                    "ping" => "Hey, I'm alive!".to_string(),
                    _ => "not implemented :(".to_string(),
                };

                if let Err(why) = interaction
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.content(content))
                    })
                    .await
                {
                    println!("Cannot respond to slash command: {}", why);
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} connected bhay", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("how th when the");
    }
}

#[group]
#[commands(ping)]
struct General;

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
        .configure(|c| c.owners(owners).prefix("xx"))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .application_id(*bot_id.as_u64())
        .await
        .expect("Client no wokey");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }
    let _ = ApplicationCommand::create_global_application_command(&http, |a| {
        a.name("ping").description("A simple ping command")
    })
    .await;

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
