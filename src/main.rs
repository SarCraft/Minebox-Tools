mod api;
mod commands;
mod util;

use poise::serenity_prelude as serenity;

/// Données partagées entre toutes les commandes (ici, le client de l'API Minebox).
pub struct Data {
    pub api: api::MineboxClient,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    // Charge le fichier .env s'il existe (ignore l'erreur si absent).
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let token = std::env::var("DISCORD_TOKEN")
        .expect("La variable d'environnement DISCORD_TOKEN doit être définie");

    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all(),
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                tracing::info!("Connecté en tant que {}", ready.user.name);
                Ok(Data {
                    api: api::MineboxClient::new(),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client
        .expect("Échec de la création du client")
        .start()
        .await
        .expect("Le bot s'est arrêté avec une erreur");
}
