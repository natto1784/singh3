mod count;
mod interactions;
use serenity::{
    async_trait,
    model::{channel::Message, event::ResumedEvent, gateway::Ready},
    prelude::*,
};
use tracing::info;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} connected bhay", ready.user.name);
    }
    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("how th when the");
    }
    async fn message(&self, ctx: Context, msg: Message) {
        let data_read = ctx.data.read().await;
        let db_client = data_read
            .get::<crate::Database>()
            .expect("Expected Database in TypeMap.")
            .clone();
        count::count(msg, db_client).await;
    }
}
