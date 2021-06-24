use serenity::builder::{CreateApplicationCommand, CreateInteractionResponseData};
use std::collections::HashMap;

pub fn general() -> Vec<CreateApplicationCommand> {
    let allah: CreateApplicationCommand = CreateApplicationCommand(HashMap::new())
        .name("allah")
        .description("acha bhay islam")
        .to_owned();
    let ping: CreateApplicationCommand = CreateApplicationCommand(HashMap::new())
        .name("ping")
        .description("Pong! bhay")
        .to_owned();
    let chut: CreateApplicationCommand = CreateApplicationCommand(HashMap::new())
        .name("chut")
        .description("yummy parantha")
        .to_owned();
    vec![allah, ping, chut]
}

pub fn responses(
    interaction_name: String,
    message: &mut CreateInteractionResponseData,
) -> &mut CreateInteractionResponseData {
    match interaction_name.as_str() {
        "allah" => message.content("suar ki chamdi".to_string()),
        _ => message.content("na hai bhay".to_string()),
    }
}
