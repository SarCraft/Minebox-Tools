//! `/recipes` — liste et détaille les recettes de craft d'un métier.

use poise::serenity_prelude as serenity;
use poise::ChoiceParameter;

use crate::util;
use crate::{Context, Error};

const LOCALE: &str = "fr";

/// Métiers de craft Minebox (la valeur API est en majuscules).
#[derive(Debug, poise::ChoiceParameter)]
pub enum Job {
    #[name = "Mineur"]
    Miner,
    #[name = "Forgeron"]
    Blacksmith,
    #[name = "Bijoutier"]
    Jeweler,
    #[name = "Forgeruneur"]
    Runeforger,
    #[name = "Alchimiste"]
    Alchemist,
    #[name = "Fermier"]
    Farmer,
    #[name = "Cuisinier"]
    Cook,
    #[name = "Pêcheur"]
    Fisherman,
    #[name = "Bûcheron"]
    Lumberjack,
    #[name = "Chasseur"]
    Hunter,
    #[name = "Cordonnier"]
    Shoemaker,
    #[name = "Tailleur"]
    Tailor,
    #[name = "Bricoleur"]
    Tinkerer,
}

impl Job {
    fn api(&self) -> &'static str {
        match self {
            Job::Miner => "MINER",
            Job::Blacksmith => "BLACKSMITH",
            Job::Jeweler => "JEWELER",
            Job::Runeforger => "RUNEFORGER",
            Job::Alchemist => "ALCHEMIST",
            Job::Farmer => "FARMER",
            Job::Cook => "COOK",
            Job::Fisherman => "FISHERMAN",
            Job::Lumberjack => "LUMBERJACK",
            Job::Hunter => "HUNTER",
            Job::Shoemaker => "SHOEMAKER",
            Job::Tailor => "TAILOR",
            Job::Tinkerer => "TINKERER",
        }
    }
}

/// Affiche les recettes de craft d'un métier (détail si une seule correspond).
#[poise::command(slash_command)]
pub async fn recipes(
    ctx: Context<'_>,
    #[description = "Métier de craft"] job: Job,
    #[description = "Filtrer par nom de recette"] search: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let api = &ctx.data().api;
    let search = search.unwrap_or_default();

    let resp = match api.recipes(job.api(), &search, LOCALE).await {
        Ok(r) => r,
        Err(e) => {
            ctx.say(format!("Impossible de récupérer les recettes ({e}).")).await?;
            return Ok(());
        }
    };

    match resp.recipes.len() {
        0 => {
            ctx.say("Aucune recette ne correspond.").await?;
        }
        1 => render_detail(ctx, &resp.recipes[0]).await?,
        _ => render_list(ctx, job.name(), &resp.recipes).await?,
    }
    Ok(())
}

/// Liste compacte quand plusieurs recettes correspondent.
async fn render_list(
    ctx: Context<'_>,
    job_name: &str,
    recipes: &[crate::api::Recipe],
) -> Result<(), Error> {
    let mut lines = String::new();
    for r in recipes.iter().take(25) {
        lines.push_str(&format!("• **{}** ({} ingrédients)\n", r.name, r.ingredients.len()));
    }
    if recipes.len() > 25 {
        lines.push_str(&format!("…et {} autres.\n", recipes.len() - 25));
    }
    lines.push_str("\n*Affinez avec `search:` pour voir le détail d'une recette.*");

    let embed = serenity::CreateEmbed::new()
        .title(format!("🛠️ Recettes — {job_name} ({})", recipes.len()))
        .description(lines)
        .colour(util::rarity_color(None));
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Détail d'une recette : item produit (image) + ingrédients (noms résolus).
async fn render_detail(ctx: Context<'_>, recipe: &crate::api::Recipe) -> Result<(), Error> {
    let api = &ctx.data().api;

    // Item produit (pour l'image et la rareté).
    let output_id = recipe.output.clone().unwrap_or_else(|| recipe.id.clone());
    let output = api.item(&output_id, LOCALE).await.ok();
    let rarity = output.as_ref().and_then(|o| o.rarity.as_deref());

    // Liste des ingrédients avec noms résolus.
    let mut ing_lines = String::new();
    for ing in &recipe.ingredients {
        let name = resolve_name(api, &ing.id).await;
        ing_lines.push_str(&format!("• **{name}** ×{}\n", ing.amount));
    }
    if ing_lines.is_empty() {
        ing_lines.push_str("*Aucun ingrédient.*");
    }

    let mut embed = serenity::CreateEmbed::new()
        .title(format!("🛠️ {}", recipe.name))
        .colour(util::rarity_color(rarity))
        .field("Ingrédients", ing_lines, false);

    let produced = if recipe.amount > 1 {
        format!("×{}", recipe.amount)
    } else {
        "×1".to_string()
    };
    embed = embed.field("Produit", produced, true);
    if let Some(job) = &recipe.job {
        embed = embed.field("Métier", job, true);
    }

    let mut reply = poise::CreateReply::default();
    // Image de l'item produit en miniature (base64 -> pièce jointe).
    if let Some(bytes) = output
        .as_ref()
        .and_then(|o| o.image.as_deref())
        .and_then(util::decode_image)
    {
        let filename = "output.png";
        embed = embed.thumbnail(format!("attachment://{filename}"));
        reply = reply.attachment(serenity::CreateAttachment::bytes(bytes, filename));
    }

    ctx.send(reply.embed(embed)).await?;
    Ok(())
}

/// Résout le nom lisible d'un item ; à défaut, embellit son identifiant.
async fn resolve_name(api: &crate::api::MineboxClient, id: &str) -> String {
    if let Ok(item) = api.item(id, LOCALE).await {
        return item.name;
    }
    util::prettify_id(id)
}
