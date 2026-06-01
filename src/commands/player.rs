//! `/player` — affiche les données d'un joueur Minebox par pseudo (ou UUID).

use poise::serenity_prelude as serenity;

use crate::util;
use crate::{Context, Error};

/// Affiche le profil Minebox d'un joueur : niveau, métiers, stats, compagnons.
#[poise::command(slash_command)]
pub async fn player(
    ctx: Context<'_>,
    #[description = "Pseudo Minecraft ou UUID"] name: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let p = match ctx.data().api.player(&name).await {
        Ok(p) => p,
        Err(_) => {
            ctx.say(format!("Aucun joueur Minebox trouvé pour « {name} »."))
                .await?;
            return Ok(());
        }
    };

    let mut embed = serenity::CreateEmbed::new()
        .title(format!("👤 {}", p.username))
        .colour(util::rarity_color(None))
        .thumbnail(format!("https://mc-heads.net/avatar/{}/128.png", p.id))
        .field("Niveau", p.level.to_string(), true)
        .field(
            "Temps de jeu",
            format!("~{} h", util::thousands(p.playtime / 60)),
            true,
        );

    if let Some(last) = p.last_connection.as_deref().and_then(util::discord_relative) {
        embed = embed.field("Vu", last, true);
    }
    if let Some(first) = p
        .first_connection
        .as_deref()
        .and_then(util::discord_relative)
    {
        embed = embed.field("Inscrit", first, true);
    }

    // Métiers (SKILLS.data) triés par XP décroissante, avec niveau calculé.
    if let Some(skills) = p.data.pointer("/SKILLS/data").and_then(|v| v.as_object()) {
        let curves = &ctx.data().skills;
        let mut jobs: Vec<(&String, i64)> = skills
            .iter()
            .map(|(k, v)| (k, v.as_i64().unwrap_or(0)))
            .collect();
        jobs.sort_by(|a, b| b.1.cmp(&a.1));
        let lines: Vec<String> = jobs
            .iter()
            .take(10)
            .map(|(job, xp)| {
                let skill = curves.get(&job.to_ascii_lowercase());
                let name = skill.map_or_else(|| util::prettify_id(job), |s| s.name.clone());
                match skill {
                    Some(s) if !s.experience_per_level.is_empty() => {
                        let lvl = util::skill_level(*xp, &s.experience_per_level);
                        format!("**{name}** — Niv. {lvl} · {} XP", util::thousands(*xp))
                    }
                    _ => format!("**{name}** — {} XP", util::thousands(*xp)),
                }
            })
            .collect();
        if !lines.is_empty() {
            embed = embed.field("Métiers", lines.join("\n"), false);
        }
    }

    // Statistiques de base attribuées.
    if let Some(base) = p
        .data
        .pointer("/ATTRIBUTED_STATS/base")
        .and_then(|v| v.as_object())
    {
        let line: Vec<String> = base
            .iter()
            .filter(|(_, v)| v.as_i64().unwrap_or(0) != 0)
            .map(|(k, v)| format!("{} {}", util::prettify_id(k), util::fmt_value(v)))
            .collect();
        if !line.is_empty() {
            embed = embed.field("Stats", line.join(" · "), false);
        }
    }

    // Compagnons (monture/familier actif et nombres).
    let companions = &p.data["COMPANIONS"];
    if companions.is_object() {
        let mount = companions
            .get("active_mount")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(util::prettify_id)
            .unwrap_or_else(|| "aucune".to_string());
        let mounts = companions.get("mounts").and_then(|v| v.as_object()).map_or(0, |m| m.len());
        let pets = companions.get("pets").and_then(|v| v.as_object()).map_or(0, |m| m.len());
        embed = embed.field(
            "Compagnons",
            format!("Monture active : **{mount}**\n🐎 {mounts} monture(s) · 🐾 {pets} familier(s)"),
            false,
        );
    }

    embed = embed.footer(serenity::CreateEmbedFooter::new(format!("UUID : {}", p.id)));
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
