//! `/roadmap` — feuille de route des sets d'équipement, du début à la fin du jeu.
//!
//! L'API `/sets` ne fournit ni niveau ni rareté ; on classe donc les sets par
//! puissance totale (somme des bonus au palier maximal) et on les découpe en
//! trois phases de progression.

use std::collections::BTreeMap;

use poise::serenity_prelude as serenity;

use crate::api::EquipmentSet;
use crate::util;
use crate::{Context, Error};

/// Phases de la roadmap, du début à la fin du jeu.
const PHASES: [(&str, &str); 3] = [
    ("🟢 Début de jeu", "Premiers sets accessibles pour démarrer."),
    ("🔵 Milieu de jeu", "Sets intermédiaires pour monter en puissance."),
    ("🟠 Fin de jeu", "Les sets les plus puissants — objectif final."),
];

/// Affiche une roadmap des sets à obtenir pour progresser vers la fin du jeu.
#[poise::command(slash_command)]
pub async fn roadmap(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let mut sets = match ctx.data().api.sets().await {
        Ok(r) => r.sets,
        Err(e) => {
            ctx.say(format!("Impossible de récupérer les sets ({e}).")).await?;
            return Ok(());
        }
    };

    if sets.is_empty() {
        ctx.say("Aucun set renvoyé par l'API.").await?;
        return Ok(());
    }

    // Classement du plus faible au plus puissant (ordre de progression).
    sets.sort_by(|a, b| a.power().cmp(&b.power()).then_with(|| a.name.cmp(&b.name)));

    // Découpe en trois phases (terciles) pour la roadmap.
    let n = sets.len();
    let cut1 = n / 3;
    let cut2 = 2 * n / 3;
    let chunks = [&sets[..cut1], &sets[cut1..cut2], &sets[cut2..]];

    let mut embed = serenity::CreateEmbed::new()
        .title("🗺️ Roadmap des sets — vers la fin du jeu")
        .colour(util::rarity_color(Some("legendary")))
        .description(format!(
            "{n} sets classés par puissance. Suis les phases de haut en bas pour \
             progresser le plus vite vers l'end-game.",
        ))
        .footer(serenity::CreateEmbedFooter::new(
            "Puissance = somme des bonus au nombre de pièces maximal",
        ));

    for ((title, hint), chunk) in PHASES.iter().zip(chunks) {
        let lines: Vec<String> = chunk.iter().map(|s| format!("• {}", set_line(s))).collect();
        // Un champ Discord est limité à 1024 caractères : on scinde une phase en
        // plusieurs champs (« … (suite) ») plutôt que de tronquer des sets.
        for (i, value) in split_fields(&lines, hint).into_iter().enumerate() {
            let name = if i == 0 {
                (*title).to_string()
            } else {
                format!("{title} (suite)")
            };
            embed = embed.field(name, value, false);
        }
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Limite Discord pour la valeur d'un champ d'embed.
const FIELD_LIMIT: usize = 1024;

/// Regroupe des lignes en valeurs de champ tenant sous la limite Discord.
/// La première valeur est préfixée par `hint` (description de la phase).
fn split_fields(lines: &[String], hint: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = format!("*{hint}*");
    for line in lines {
        // +1 pour le saut de ligne ajouté avant la ligne.
        if current.len() + 1 + line.len() > FIELD_LIMIT {
            fields.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
    }
    if !current.is_empty() {
        fields.push(current);
    }
    fields
}

/// Une ligne de set : nom, pièces du palier max, et ses 2 meilleurs bonus.
fn set_line(s: &EquipmentSet) -> String {
    let Some((pieces, stats)) = s.max_tier() else {
        return format!("**{}**", s.name);
    };
    format!(
        "**{}** — {pieces} pièces · {}",
        s.name,
        top_bonuses(stats, 2)
    )
}

/// Les `n` plus gros bonus d'un palier, formatés « Strength +20 ».
fn top_bonuses(stats: &BTreeMap<String, i64>, n: usize) -> String {
    let mut entries: Vec<(&String, &i64)> = stats.iter().collect();
    entries.sort_by(|a, b| b.1.cmp(a.1));
    entries
        .into_iter()
        .take(n)
        .map(|(stat, val)| format!("{} +{val}", util::prettify_id(stat)))
        .collect::<Vec<_>>()
        .join(", ")
}
