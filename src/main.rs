use poise::serenity_prelude as serenity;

/// Données partagées entre toutes les commandes.
struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Répond avec « Pong ! » et la latence du bot.
#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let latency = ctx.ping().await;
    ctx.say(format!("🏓 Pong ! Latence : {} ms", latency.as_millis()))
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    // Charge le fichier .env s'il existe (ignore l'erreur si absent).
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let token = std::env::var("DISCORD_TOKEN")
        .expect("La variable d'environnement DISCORD_TOKEN doit être définie");

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                tracing::info!("Connecté en tant que {}", ready.user.name);
                Ok(Data {})
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
