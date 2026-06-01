//! `/market` — consultation du marché (prix, bazar, hôtel des ventes).
//!
//! L'API publique renvoie souvent un corps vide pour ces routes (marché vide
//! ou réservé), ce qui est géré proprement par un message dédié.

use poise::serenity_prelude as serenity;

use crate::util;
use crate::{Context, Error};

/// Commande parente : utilisez une sous-commande (`prices`, `bazaar`, `auction`).
#[poise::command(slash_command, subcommands("prices", "bazaar", "auction"))]
pub async fn market(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Statistiques de prix d'un item (achat / vente / direct).
#[poise::command(slash_command)]
pub async fn prices(
    ctx: Context<'_>,
    #[description = "Identifiant de l'item (ex. ingot_opale)"] item_id: String,
    #[description = "Période (ex. 7d, 30d)"] period: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let period = period.unwrap_or_else(|| "7d".to_string());
    match ctx.data().api.market_prices(&item_id, &period).await {
        Ok(v) => {
            let mut embed = serenity::CreateEmbed::new()
                .title(format!("💰 Prix — {}", util::prettify_id(&item_id)))
                .colour(util::rarity_color(None))
                .footer(serenity::CreateEmbedFooter::new(format!("période : {period}")));
            for key in ["BUY", "SELL", "DIRECT"] {
                let label = match key {
                    "BUY" => "Achat",
                    "SELL" => "Vente",
                    _ => "Direct",
                };
                let val = v.get(key).map(util::fmt_value).unwrap_or_else(|| "—".into());
                embed = embed.field(label, val, true);
            }
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            ctx.say(format!("Aucune donnée de prix pour « {item_id} » ({e}).")).await?;
        }
    }
    Ok(())
}

/// Offres du bazar (achat/vente entre joueurs) pour un item.
#[poise::command(slash_command)]
pub async fn bazaar(
    ctx: Context<'_>,
    #[description = "Identifiant de l'item"] item_id: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    match ctx.data().api.market_bazaar(&item_id, 10).await {
        Ok(v) => {
            let embed = serenity::CreateEmbed::new()
                .title(format!("🏪 Bazar — {}", util::prettify_id(&item_id)))
                .colour(util::rarity_color(None))
                .description(render_listings(&v));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(_) => {
            ctx.say(format!("Aucune offre de bazar pour « {item_id} » actuellement."))
                .await?;
        }
    }
    Ok(())
}

/// Annonces récentes de l'hôtel des ventes.
#[poise::command(slash_command)]
pub async fn auction(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    match ctx.data().api.market_auction(10).await {
        Ok(v) => {
            let embed = serenity::CreateEmbed::new()
                .title("⚖️ Hôtel des ventes")
                .colour(util::rarity_color(None))
                .description(render_listings(&v));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(_) => {
            ctx.say("Aucune annonce à l'hôtel des ventes actuellement.").await?;
        }
    }
    Ok(())
}

/// Rend une liste d'annonces JSON (forme variable) de façon compacte.
fn render_listings(v: &serde_json::Value) -> String {
    // L'API peut renvoyer un tableau, ou un objet contenant un tableau.
    let array = v.as_array().cloned().or_else(|| {
        v.as_object().and_then(|o| {
            for key in ["listings", "items", "results", "data", "orders"] {
                if let Some(serde_json::Value::Array(a)) = o.get(key) {
                    return Some(a.clone());
                }
            }
            None
        })
    });

    match array {
        Some(items) if !items.is_empty() => {
            let mut out = String::new();
            for it in items.iter().take(10) {
                out.push_str(&format!("• {}\n", util::fmt_value(it)));
            }
            out
        }
        _ => "Aucune donnée disponible.".to_string(),
    }
}
