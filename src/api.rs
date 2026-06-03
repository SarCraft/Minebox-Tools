//! Client de l'API publique Minebox (https://api.minebox.co).

use serde::{Deserialize, Deserializer};

const BASE_URL: &str = "https://api.minebox.co";

/// Désérialise une séquence en la traitant comme vide si le champ vaut `null`.
///
/// L'API Minebox renvoie parfois `null` (et non `[]` ou un champ absent) pour
/// une liste vide ; `#[serde(default)]` ne suffit pas car un `null` présent est
/// quand même transmis au désérialiseur de `Vec`.
fn null_as_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

/// Client HTTP réutilisable vers l'API Minebox.
#[derive(Clone)]
pub struct MineboxClient {
    http: reqwest::Client,
}

impl MineboxClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .user_agent("minebox-bot (Discord)")
            .build()
            .expect("client reqwest");
        Self { http }
    }

    /// GET `path` (commençant par `/`) et désérialise le JSON en `T`.
    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, ApiError> {
        let url = format!("{BASE_URL}{path}");
        let resp = self.http.get(&url).query(query).send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(ApiError::Status(status.as_u16()));
        }
        if bytes.is_empty() {
            return Err(ApiError::Empty);
        }
        serde_json::from_slice(&bytes).map_err(ApiError::Parse)
    }

    /// Liste du bestiaire (utilisé pour l'autocomplétion et la recherche).
    pub async fn bestiary(
        &self,
        search: &str,
        locale: &str,
    ) -> Result<BestiaryList, ApiError> {
        self.get(
            "/bestiary",
            &[
                ("search", search.to_string()),
                ("pageSize", "25".to_string()),
                ("locale", locale.to_string()),
            ],
        )
        .await
    }

    /// Détail d'une créature avec ses loots.
    pub async fn creature(&self, id: &str, locale: &str) -> Result<Creature, ApiError> {
        self.get(
            &format!("/bestiary/{id}"),
            &[("locale", locale.to_string())],
        )
        .await
    }

    /// Recettes de craft pour un métier donné.
    pub async fn recipes(
        &self,
        job: &str,
        search: &str,
        locale: &str,
    ) -> Result<RecipesResp, ApiError> {
        self.get(
            "/recipes",
            &[
                ("job", job.to_string()),
                ("search", search.to_string()),
                ("locale", locale.to_string()),
            ],
        )
        .await
    }

    /// Détail d'un item (nom, image base64, rareté…).
    pub async fn item(&self, id: &str, locale: &str) -> Result<Item, ApiError> {
        self.get(&format!("/item/{id}"), &[("locale", locale.to_string())])
            .await
    }

    /// Statistiques de prix marché d'un item.
    pub async fn market_prices(
        &self,
        item_id: &str,
        period: &str,
    ) -> Result<serde_json::Value, ApiError> {
        self.get(
            "/market/prices",
            &[
                ("item_id", item_id.to_string()),
                ("period", period.to_string()),
            ],
        )
        .await
    }

    /// Annonces de l'hôtel des ventes (auction house).
    pub async fn market_auction(&self, limit: u32) -> Result<serde_json::Value, ApiError> {
        self.get("/market/auction", &[("limit", limit.to_string())])
            .await
    }

    /// Offres du bazar pour un item.
    pub async fn market_bazaar(
        &self,
        item_id: &str,
        limit: u32,
    ) -> Result<serde_json::Value, ApiError> {
        self.get(
            "/market/bazaar",
            &[
                ("item_id", item_id.to_string()),
                ("limit", limit.to_string()),
            ],
        )
        .await
    }

    /// Données complètes d'un joueur (pseudo ou UUID).
    pub async fn player(&self, identifier: &str) -> Result<Player, ApiError> {
        self.get(&format!("/data/{identifier}"), &[]).await
    }

    /// Liste des métiers avec leur courbe d'XP par niveau.
    pub async fn skills(&self, locale: &str) -> Result<SkillsResp, ApiError> {
        self.get("/skills", &[("locale", locale.to_string())]).await
    }

    /// Informations d'une guilde (nom ou UUID).
    pub async fn guild(&self, identifier: &str) -> Result<Guild, ApiError> {
        self.get(&format!("/guild/{identifier}"), &[]).await
    }

    /// Liste de tous les sets d'équipement avec leurs bonus.
    pub async fn sets(&self) -> Result<SetsResp, ApiError> {
        self.get("/sets", &[]).await
    }
}

// ---------------------------------------------------------------------------
// Types de réponse
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct BestiaryList {
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub creatures: Vec<CreatureSummary>,
}

#[derive(Debug, Deserialize)]
pub struct CreatureSummary {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub family_name: Option<String>,
    #[serde(default)]
    pub level: i64,
    #[serde(default)]
    pub level_max: i64,
}

#[derive(Debug, Deserialize)]
pub struct Creature {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub family_name: Option<String>,
    #[serde(default)]
    pub level: i64,
    #[serde(default)]
    pub level_max: i64,
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub health: Vec<i64>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub zones: Option<Vec<String>>,
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub drops: Vec<Drop>,
    #[serde(default)]
    pub stats: Option<std::collections::BTreeMap<String, Vec<i64>>>,
}

#[derive(Debug, Deserialize)]
pub struct Drop {
    pub item_id: String,
    pub name: String,
    /// Image PNG encodée en base64 (pas une URL).
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub amount: Vec<i64>,
    #[serde(default)]
    pub chance: f64,
    #[serde(default)]
    pub item: Option<ItemBrief>,
}

#[derive(Debug, Deserialize)]
pub struct ItemBrief {
    #[serde(default)]
    pub rarity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecipesResp {
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub recipes: Vec<Recipe>,
    #[serde(default)]
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub job: Option<String>,
    #[serde(default)]
    pub amount: i64,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub ingredients: Vec<Ingredient>,
}

#[derive(Debug, Deserialize)]
pub struct Ingredient {
    pub id: String,
    #[serde(default)]
    pub amount: i64,
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub rarity: Option<String>,
    /// Image PNG encodée en base64.
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub level: Option<i64>,
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub level: i64,
    #[serde(default)]
    pub xp: i64,
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub members: Vec<GuildMember>,
}

impl Guild {
    /// Nombre de membres actuellement connectés.
    pub fn online_count(&self) -> usize {
        self.members.iter().filter(|m| m.online).count()
    }
}

#[derive(Debug, Deserialize)]
pub struct GuildMember {
    pub username: String,
    #[serde(default)]
    pub online: bool,
    #[serde(default)]
    pub is_owner: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetsResp {
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub sets: Vec<EquipmentSet>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EquipmentSet {
    pub id: String,
    pub name: String,
    /// Bonus par nombre de pièces portées : `{ "2": { "STRENGTH": 5, … }, … }`.
    #[serde(default)]
    pub bonuses: std::collections::BTreeMap<String, std::collections::BTreeMap<String, i64>>,
}

impl EquipmentSet {
    /// Palier maximal (le plus grand nombre de pièces) et ses bonus.
    pub fn max_tier(&self) -> Option<(u32, &std::collections::BTreeMap<String, i64>)> {
        self.bonuses
            .iter()
            .filter_map(|(k, v)| k.parse::<u32>().ok().map(|n| (n, v)))
            .max_by_key(|(n, _)| *n)
    }

    /// Somme des bonus au palier maximal : approxime la puissance du set.
    pub fn power(&self) -> i64 {
        self.max_tier()
            .map(|(_, stats)| stats.values().sum())
            .unwrap_or(0)
    }
}

#[derive(Debug, Deserialize)]
pub struct SkillsResp {
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub skills: Vec<Skill>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Skill {
    pub id: String,
    pub name: String,
    /// Coût d'XP cumulé par niveau (`[0]` = niveau 1, cumul ensuite).
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub experience_per_level: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub level: i64,
    #[serde(default)]
    pub playtime: i64,
    #[serde(default)]
    pub first_connection: Option<String>,
    #[serde(default)]
    pub last_connection: Option<String>,
    #[serde(default)]
    pub online: bool,
    #[serde(default)]
    pub data: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Erreurs
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ApiError {
    Http(reqwest::Error),
    Status(u16),
    /// Réponse 200 mais corps vide (fréquent sur l'API Minebox quand il n'y a
    /// pas de données : item introuvable, marché vide, joueur inconnu…).
    Empty,
    Parse(serde_json::Error),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Http(e) => write!(f, "erreur réseau : {e}"),
            ApiError::Status(code) => write!(f, "l'API a répondu HTTP {code}"),
            ApiError::Empty => write!(f, "aucune donnée renvoyée"),
            ApiError::Parse(e) => write!(f, "réponse illisible : {e}"),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::Http(e)
    }
}
