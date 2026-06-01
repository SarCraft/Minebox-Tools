//! Commandes slash du bot.

mod bestiary;
pub mod guild;
mod market;
mod player;
mod recipes;

use crate::{Data, Error};

/// Renvoie la liste de toutes les commandes à enregistrer auprès de Discord.
pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        ping(),
        bestiary::bestiary(),
        recipes::recipes(),
        market::market(),
        player::player(),
        guild::guild(),
    ]
}

/// Répond avec « Pong ! » et la latence du bot.
#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: crate::Context<'_>) -> Result<(), Error> {
    let latency = ctx.ping().await;
    ctx.say(format!("🏓 Pong ! Latence : {} ms", latency.as_millis()))
        .await?;
    Ok(())
}
