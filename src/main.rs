mod api;
mod commands;
mod stats;
mod util;

use poise::serenity_prelude as serenity;

/// Données partagées entre toutes les commandes.
pub struct Data {
    pub api: api::MineboxClient,
    /// Métiers indexés par identifiant minuscule (nom localisé + courbe d'XP).
    pub skills: std::collections::HashMap<String, api::Skill>,
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

                let api = api::MineboxClient::new();
                // Pré-charge les courbes de niveaux des métiers (pour /player).
                let skills = match api.skills("fr").await {
                    Ok(resp) => resp
                        .skills
                        .into_iter()
                        .map(|s| (s.id.to_ascii_lowercase(), s))
                        .collect(),
                    Err(e) => {
                        tracing::warn!("Impossible de charger les métiers : {e}");
                        std::collections::HashMap::new()
                    }
                };

                // Guilde suivie (statut + dashboards). Défaut « S7ven ».
                let guild_name = std::env::var("GUILD_PRESENCE")
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "S7ven".to_string());

                // Tableau de bord « live » optionnel (si STATS_CHANNEL_ID est défini).
                // Les joueurs suivis sont les membres de la guilde ; `STATS_PLAYERS`
                // (ou la liste par défaut) sert de repli si l'API guilde échoue.
                if let Some(channel_id) = std::env::var("STATS_CHANNEL_ID")
                    .ok()
                    .and_then(|s| s.trim().parse::<u64>().ok())
                {
                    let fallback = std::env::var("STATS_PLAYERS")
                        .ok()
                        .map(|s| {
                            s.split(',')
                                .map(|p| p.trim().to_string())
                                .filter(|p| !p.is_empty())
                                .collect::<Vec<_>>()
                        })
                        .filter(|v| !v.is_empty())
                        .unwrap_or_else(stats::default_players);

                    stats::spawn(
                        ctx.clone(),
                        api.clone(),
                        skills.clone(),
                        channel_id,
                        guild_name.clone(),
                        fallback,
                    );
                }

                stats::spawn_presence(ctx.clone(), api.clone(), guild_name.clone());

                // Dashboard de guilde dans un salon (si GUILD_CHANNEL_ID est défini).
                if let Some(guild_channel) = std::env::var("GUILD_CHANNEL_ID")
                    .ok()
                    .and_then(|s| s.trim().parse::<u64>().ok())
                {
                    stats::spawn_guild(ctx.clone(), api.clone(), guild_name, guild_channel);
                }

                Ok(Data { api, skills })
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
