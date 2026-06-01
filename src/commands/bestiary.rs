//! `/bestiary` — recherche une créature et affiche ses loots avec images.

use poise::serenity_prelude as serenity;

use crate::util;
use crate::{Context, Error};

const LOCALE: &str = "fr";
/// Discord limite à 10 embeds par message : 1 pour la créature + 9 loots max.
const MAX_LOOT_EMBEDS: usize = 9;

/// Autocomplétion : propose les créatures dont le nom correspond à la saisie.
async fn autocomplete_creature(
    ctx: Context<'_>,
    partial: &str,
) -> Vec<serenity::AutocompleteChoice> {
    let api = &ctx.data().api;
    match api.bestiary(partial, LOCALE).await {
        Ok(list) => list
            .creatures
            .into_iter()
            .take(25)
            .map(|c| {
                let family = c.family_name.unwrap_or_default();
                let lvl = if c.level_max > c.level {
                    format!("Nv.{}–{}", c.level, c.level_max)
                } else {
                    format!("Nv.{}", c.level)
                };
                let label = format!("{} • {family} {lvl}", c.name);
                let label: String = label.chars().take(100).collect();
                serenity::AutocompleteChoice::new(label, c.id)
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Affiche une créature du bestiaire Minebox : infos et loots (avec images).
#[poise::command(slash_command)]
pub async fn bestiary(
    ctx: Context<'_>,
    #[description = "Nom de la créature"]
    #[autocomplete = "autocomplete_creature"]
    creature: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let api = &ctx.data().api;

    // Si l'utilisateur a tapé un nom sans choisir dans la liste, on tente une
    // recherche pour retrouver l'identifiant correspondant.
    let id = match api.creature(&creature, LOCALE).await {
        Ok(c) => return render(ctx, c).await,
        Err(_) => match api.bestiary(&creature, LOCALE).await {
            Ok(list) if !list.creatures.is_empty() => list.creatures[0].id.clone(),
            _ => {
                ctx.say(format!("Aucune créature trouvée pour « {creature} »."))
                    .await?;
                return Ok(());
            }
        },
    };

    match api.creature(&id, LOCALE).await {
        Ok(c) => render(ctx, c).await,
        Err(e) => {
            ctx.say(format!("Impossible de récupérer la créature ({e})."))
                .await?;
            Ok(())
        }
    }
}

/// Construit et envoie le message (embed créature + embeds de loots illustrés).
async fn render(ctx: Context<'_>, c: crate::api::Creature) -> Result<(), Error> {
    let mut desc = String::new();
    if let Some(fam) = &c.family_name {
        desc.push_str(&format!("**Famille :** {fam}\n"));
    }
    let lvl = if c.level_max > c.level {
        format!("{}–{}", c.level, c.level_max)
    } else {
        c.level.to_string()
    };
    desc.push_str(&format!("**Niveau :** {lvl}\n"));
    if !c.health.is_empty() {
        desc.push_str(&format!("**Vie :** ❤️ {}\n", util::range(&c.health)));
    }
    if let Some(zones) = &c.zones {
        if !zones.is_empty() {
            desc.push_str(&format!("**Zones :** {}\n", zones.join(", ")));
        }
    }
    if let Some(stats) = &c.stats {
        if !stats.is_empty() {
            let line: Vec<String> = stats
                .iter()
                .map(|(k, v)| format!("{k} {}", util::range(v)))
                .collect();
            desc.push_str(&format!("**Stats :** {}\n", line.join(" · ")));
        }
    }

    let mut lead = serenity::CreateEmbed::new()
        .title(format!("🐾 {}", c.name))
        .description(desc)
        .colour(util::rarity_color(None));
    if let Some(img) = &c.image {
        lead = lead.thumbnail(img);
    }

    // Résumé textuel des loots dans l'embed principal.
    if c.drops.is_empty() {
        lead = lead.field("Loots", "Aucun loot connu.", false);
    } else {
        let mut summary = String::new();
        for d in &c.drops {
            let dot = util::rarity_dot(d.item.as_ref().and_then(|i| i.rarity.as_deref()));
            let amount = if d.amount.is_empty() {
                String::new()
            } else {
                format!(" x{}", util::range(&d.amount))
            };
            summary.push_str(&format!("{dot} **{}**{amount} — {:.0}%\n", d.name, d.chance));
        }
        // Le champ d'embed est limité à 1024 caractères.
        summary.truncate(1020);
        lead = lead.field(format!("Loots ({})", c.drops.len()), summary, false);
    }

    let mut reply = poise::CreateReply::default().embed(lead);

    // Un embed illustré par loot (jusqu'à la limite Discord).
    for (i, d) in c.drops.iter().take(MAX_LOOT_EMBEDS).enumerate() {
        let rarity = d.item.as_ref().and_then(|it| it.rarity.as_deref());
        let amount = if d.amount.is_empty() {
            String::new()
        } else {
            format!("Quantité : {}\n", util::range(&d.amount))
        };
        let mut embed = serenity::CreateEmbed::new()
            .title(format!("{} {}", util::rarity_dot(rarity), d.name))
            .description(format!("{amount}Chance : {:.1}%", d.chance))
            .colour(util::rarity_color(rarity));

        // Décode l'image base64 et l'attache, puis la référence en miniature.
        if let Some(bytes) = d.image.as_deref().and_then(util::decode_image) {
            let filename = format!("loot_{i}.png");
            embed = embed.thumbnail(format!("attachment://{filename}"));
            reply = reply.attachment(serenity::CreateAttachment::bytes(bytes, filename));
        }
        reply = reply.embed(embed);
    }

    ctx.send(reply).await?;
    Ok(())
}
