mod interactions;
use serenity::{
    async_trait,
    model::{
        event::ResumedEvent,
        gateway::Ready,
        interactions::{
            ApplicationCommand, Interaction, InteractionData, InteractionResponseType,
            InteractionType,
        },
    },
    prelude::*,
};
use tracing::info;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} connected bhay", ready.user.name);
        let _ = ApplicationCommand::create_global_application_commands(&ctx.http, |commands| {
            commands.set_application_commands(interactions::general())
        })
        .await;
    }
    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("how th when the");
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if interaction.kind == InteractionType::ApplicationCommand {
            if let Some(InteractionData::ApplicationCommand(data)) = interaction.data.as_ref() {
                if let Err(why) = interaction
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| interactions::responses(data.name.to_string(), message))
                    })
                    .await
                {
                    println!("Cannot respond to slash command: {}", why);
                }
            }
        }
    }
}
