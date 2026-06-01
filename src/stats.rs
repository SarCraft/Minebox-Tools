//! Tableau de bord « live » : un message dans un salon, mis à jour chaque minute
//! avec les stats d'une liste de joueurs.

use std::collections::HashMap;
use std::time::Duration;

use poise::serenity_prelude as serenity;

use crate::api::{MineboxClient, Skill};
use crate::util;

/// Intervalle de rafraîchissement.
const INTERVAL: Duration = Duration::from_secs(60);
/// Titre servant à retrouver/identifier le message du tableau de bord.
const TITLE: &str = "📊 Stats Minebox — Live";
/// Nombre de métiers affichés par joueur.
const TOP_JOBS: usize = 3;

/// Joueurs suivis par défaut (si `STATS_PLAYERS` n'est pas défini).
pub fn default_players() -> Vec<String> {
    ["MasteRX_", "Nasara__", "Finkeleb", "Ayzziixx", "Doriko79", "Nathan201204"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Met à jour le statut du bot (« petite bulle ») avec le nombre de membres
/// connectés d'une guilde, rafraîchi chaque minute.
pub fn spawn_presence(ctx: serenity::Context, api: MineboxClient, guild_name: String) {
    tokio::spawn(async move {
        tracing::info!("Présence : suivi des connectés de la guilde « {guild_name} »");
        loop {
            let activity = match api.guild(&guild_name).await {
                Ok(g) => {
                    let online = g.online_count();
                    serenity::ActivityData::custom(format!(
                        "🟢 {online} connecté(s) dans {}",
                        g.name
                    ))
                }
                Err(_) => serenity::ActivityData::custom(format!(
                    "Guilde {guild_name} indisponible"
                )),
            };
            ctx.set_presence(Some(activity), serenity::OnlineStatus::Online);
            tokio::time::sleep(INTERVAL).await;
        }
    });
}

/// Lance la boucle de mise à jour en tâche de fond.
pub fn spawn(
    ctx: serenity::Context,
    api: MineboxClient,
    skills: HashMap<String, Skill>,
    channel_id: u64,
    players: Vec<String>,
) {
    tokio::spawn(async move {
        let channel = serenity::ChannelId::new(channel_id);
        let bot_id = ctx.cache.current_user().id;

        // Réutilise un message existant du bot s'il y en a un (évite les doublons
        // à chaque redémarrage).
        let mut message_id = find_existing(&ctx, channel, bot_id).await;
        tracing::info!(
            "Tableau de bord stats actif dans le salon {channel_id} ({} joueurs)",
            players.len()
        );

        loop {
            let embed = build_embed(&api, &skills, &players).await;

            message_id = match message_id {
                Some(id) => {
                    let edit = serenity::EditMessage::new().embed(embed.clone());
                    match channel.edit_message(&ctx.http, id, edit).await {
                        Ok(_) => Some(id),
                        // Message supprimé entre-temps : on en renvoie un nouveau.
                        Err(_) => send_new(&ctx, channel, embed).await,
                    }
                }
                None => send_new(&ctx, channel, embed).await,
            };

            tokio::time::sleep(INTERVAL).await;
        }
    });
}

/// Envoie un nouveau message et renvoie son identifiant.
async fn send_new(
    ctx: &serenity::Context,
    channel: serenity::ChannelId,
    embed: serenity::CreateEmbed,
) -> Option<serenity::MessageId> {
    let msg = serenity::CreateMessage::new().embed(embed);
    match channel.send_message(&ctx.http, msg).await {
        Ok(m) => Some(m.id),
        Err(e) => {
            tracing::warn!("Envoi du message de stats impossible : {e}");
            None
        }
    }
}

/// Cherche un message déjà posté par le bot (reconnu à son titre d'embed).
async fn find_existing(
    ctx: &serenity::Context,
    channel: serenity::ChannelId,
    bot_id: serenity::UserId,
) -> Option<serenity::MessageId> {
    let builder = serenity::GetMessages::new().limit(50);
    let messages = channel.messages(&ctx.http, builder).await.ok()?;
    messages
        .into_iter()
        .find(|m| {
            m.author.id == bot_id
                && m.embeds
                    .first()
                    .and_then(|e| e.title.as_deref())
                    .is_some_and(|t| t == TITLE)
        })
        .map(|m| m.id)
}

/// Construit l'embed du tableau de bord (un champ par joueur).
async fn build_embed(
    api: &MineboxClient,
    skills: &HashMap<String, Skill>,
    players: &[String],
) -> serenity::CreateEmbed {
    let mut embed = serenity::CreateEmbed::new()
        .title(TITLE)
        .colour(util::rarity_color(None))
        .timestamp(serenity::Timestamp::now())
        .footer(serenity::CreateEmbedFooter::new(
            "Mise à jour automatique toutes les minutes",
        ));

    for name in players {
        match api.player(name).await {
            Ok(p) => {
                let dot = if p.online { "🟢" } else { "⚫" };
                let mut value = format!(
                    "Niveau **{}** · ~{} h\n",
                    p.level,
                    util::thousands(p.playtime / 60)
                );

                if p.online {
                    value.push_str("🎮 *En ligne*\n");
                } else if let Some(seen) =
                    p.last_connection.as_deref().and_then(util::discord_relative)
                {
                    value.push_str(&format!("Vu {seen}\n"));
                }

                let jobs = top_jobs(&p, skills);
                if !jobs.is_empty() {
                    value.push_str(&jobs);
                }

                embed = embed.field(format!("{dot} {}", p.username), value, true);
            }
            Err(_) => {
                embed = embed.field(format!("⚫ {name}"), "*Introuvable*", true);
            }
        }
    }

    embed
}

/// Renvoie les `TOP_JOBS` métiers du joueur (niveau calculé), triés par XP.
fn top_jobs(p: &crate::api::Player, skills: &HashMap<String, Skill>) -> String {
    let Some(map) = p.data.pointer("/SKILLS/data").and_then(|v| v.as_object()) else {
        return String::new();
    };
    let mut jobs: Vec<(&String, i64)> = map
        .iter()
        .map(|(k, v)| (k, v.as_i64().unwrap_or(0)))
        .collect();
    jobs.sort_by(|a, b| b.1.cmp(&a.1));

    jobs.iter()
        .take(TOP_JOBS)
        .map(|(job, xp)| {
            let skill = skills.get(&job.to_ascii_lowercase());
            let label = skill.map_or_else(|| util::prettify_id(job), |s| s.name.clone());
            match skill {
                Some(s) if !s.experience_per_level.is_empty() => {
                    let lvl = util::skill_level(*xp, &s.experience_per_level);
                    format!("`{label} {lvl}`")
                }
                _ => format!("`{label}`"),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
