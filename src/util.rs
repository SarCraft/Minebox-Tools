//! Petits utilitaires partagés par les commandes.

use base64::Engine;
use poise::serenity_prelude as serenity;

/// Couleur d'embed associée à une rareté Minebox.
pub fn rarity_color(rarity: Option<&str>) -> serenity::Colour {
    let r = rarity.unwrap_or("").to_ascii_lowercase();
    let hex = match r.as_str() {
        "trash" => 0x6E6E6E,
        "common" => 0xB0B0B0,
        "uncommon" => 0x4CAF50,
        "rare" => 0x2196F3,
        "epic" => 0x9C27B0,
        "legendary" => 0xFF9800,
        "mythic" => 0xE91E63,
        "prototype" | "contraband" => 0x00BCD4,
        _ => 0x5865F2, // blurple par défaut
    };
    serenity::Colour::new(hex)
}

/// Petit cercle coloré (emoji) pour symboliser la rareté dans le texte.
pub fn rarity_dot(rarity: Option<&str>) -> &'static str {
    match rarity.unwrap_or("").to_ascii_lowercase().as_str() {
        "trash" => "⚪",
        "common" => "⚪",
        "uncommon" => "🟢",
        "rare" => "🔵",
        "epic" => "🟣",
        "legendary" => "🟠",
        "mythic" => "🔴",
        "prototype" | "contraband" => "🩵",
        _ => "▫️",
    }
}

/// Décode une image base64 (éventuellement préfixée d'un data-URI) en octets PNG.
pub fn decode_image(b64: &str) -> Option<Vec<u8>> {
    let data = b64.split(',').last().unwrap_or(b64).trim();
    base64::engine::general_purpose::STANDARD.decode(data).ok()
}

/// Formate un intervalle `[min, max]` : « 5 » si égaux, sinon « 5–8 ».
pub fn range(v: &[i64]) -> String {
    match v {
        [a] => a.to_string(),
        [a, b, ..] if a == b => a.to_string(),
        [a, b, ..] => format!("{a}–{b}"),
        _ => "?".to_string(),
    }
}

/// Ajoute des séparateurs de milliers : 1234567 -> « 1 234 567 ».
pub fn thousands(n: i64) -> String {
    let neg = n < 0;
    let digits: Vec<char> = n.unsigned_abs().to_string().chars().collect();
    let mut out = String::new();
    for (i, c) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push('\u{202f}'); // espace fine insécable
        }
        out.push(*c);
    }
    if neg {
        format!("-{out}")
    } else {
        out
    }
}

/// Transforme un identifiant `red_wood_superplaque` en « Red Wood Superplaque ».
pub fn prettify_id(id: &str) -> String {
    id.split(['_', '-'])
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Rend une valeur JSON quelconque en texte court et lisible.
pub fn fmt_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "—".to_string(),
        serde_json::Value::Bool(b) => if *b { "oui" } else { "non" }.to_string(),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                thousands(i)
            } else {
                n.to_string()
            }
        }
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(a) => format!("{} élément(s)", a.len()),
        serde_json::Value::Object(o) => o
            .iter()
            .map(|(k, val)| format!("{k}: {}", fmt_value(val)))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// Niveau atteint pour une XP donnée selon la courbe `experience_per_level`.
pub fn skill_level(xp: i64, curve: &[i64]) -> i64 {
    skill_progress(xp, curve).level
}

/// Progression détaillée d'un métier pour une XP donnée.
///
/// `curve[0]` correspond au niveau 1 (0 XP) ; chaque entrée suivante est le coût
/// (incrémental) pour atteindre le niveau correspondant. On cumule jusqu'à
/// dépasser `xp` pour en déduire le niveau et la progression vers le suivant.
pub struct SkillProgress {
    pub level: i64,
    /// Pourcentage parcouru dans le niveau courant (0–100).
    pub percent: f64,
    /// XP déjà acquise dans le niveau courant.
    pub into_level: i64,
    /// XP totale nécessaire pour passer au niveau suivant.
    pub level_span: i64,
}

pub fn skill_progress(xp: i64, curve: &[i64]) -> SkillProgress {
    let mut level = 1;
    let mut cumulative = 0i64;
    let mut level_start = 0i64;
    for (i, cost) in curve.iter().enumerate().skip(1) {
        cumulative += cost;
        if xp >= cumulative {
            level = i as i64 + 1;
            level_start = cumulative;
        } else {
            let span = cumulative - level_start;
            let into = xp - level_start;
            let percent = if span > 0 {
                (into as f64 / span as f64) * 100.0
            } else {
                0.0
            };
            return SkillProgress {
                level,
                percent,
                into_level: into,
                level_span: span,
            };
        }
    }
    // Niveau maximum atteint.
    SkillProgress {
        level,
        percent: 100.0,
        into_level: 0,
        level_span: 0,
    }
}

/// Convertit une date ISO-8601 en timestamp Discord relatif (`<t:…:R>`).
pub fn discord_relative(iso: &str) -> Option<String> {
    let dt = chrono::DateTime::parse_from_rfc3339(iso).ok()?;
    Some(format!("<t:{}:R>", dt.timestamp()))
}
