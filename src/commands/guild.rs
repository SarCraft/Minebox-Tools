//! `/guild` — informations d'une guilde Minebox.

use poise::serenity_prelude as serenity;

use crate::util;
use crate::{Context, Error};

/// Préfixe du titre de l'embed (sert aussi à repérer le message du dashboard).
pub const TITLE_PREFIX: &str = "🛡️";

/// Affiche les infos d'une guilde : niveau, XP, membres et connectés.
#[poise::command(slash_command)]
pub async fn guild(
    ctx: Context<'_>,
    #[description = "Nom de la guilde (ou UUID)"] name: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let g = match ctx.data().api.guild(&name).await {
        Ok(g) => g,
        Err(_) => {
            ctx.say(format!("Aucune guilde trouvée pour « {name} »."))
                .await?;
            return Ok(());
        }
    };

    ctx.send(poise::CreateReply::default().embed(build_embed(&g)))
        .await?;
    Ok(())
}

/// Construit l'embed d'une guilde (réutilisé par la commande et le dashboard).
pub fn build_embed(g: &crate::api::Guild) -> serenity::CreateEmbed {
    let online = g.online_count();
    let total = g.members.len();

    // Membres triés : chef en tête, puis les connectés.
    let mut members = g.members.iter().collect::<Vec<_>>();
    members.sort_by_key(|m| (!m.is_owner, !m.online, m.username.to_lowercase()));
    let list: String = members
        .iter()
        .take(40)
        .map(|m| {
            let dot = if m.online { "🟢" } else { "⚫" };
            let crown = if m.is_owner { " 👑" } else { "" };
            format!("{dot} {}{crown}", m.username)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut embed = serenity::CreateEmbed::new()
        .title(format!("{TITLE_PREFIX} {}", g.name))
        .colour(util::rarity_color(None))
        .field("Niveau", g.level.to_string(), true)
        .field("XP", util::thousands(g.xp), true)
        .field(
            "Membres",
            format!("🟢 {online} en ligne · {total} au total"),
            true,
        );

    if !list.is_empty() {
        embed = embed.field("Liste des membres", list, false);
    }
    embed
        .timestamp(serenity::Timestamp::now())
        .footer(serenity::CreateEmbedFooter::new(format!("UUID : {}", g.id)))
}
