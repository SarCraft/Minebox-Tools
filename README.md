# Minebox Bot

Bot Discord écrit en Rust avec [Serenity](https://github.com/serenity-rs/serenity) et [Poise](https://github.com/serenity-rs/poise).

## Prérequis

- [Rust](https://rustup.rs/) (édition 2021)
- Un bot Discord créé sur le [portail développeur](https://discord.com/developers/applications) avec son token

## Configuration

1. Copiez le fichier d'exemple d'environnement :

   ```sh
   cp .env.example .env
   ```

2. Renseignez votre token dans `.env` :

   ```env
   DISCORD_TOKEN=votre_token_ici
   ```

## Lancer le bot

```sh
cargo run
```

Au démarrage, les commandes slash sont enregistrées globalement (la propagation peut prendre jusqu'à une heure côté Discord).

## Commandes

Toutes les données proviennent de l'[API publique Minebox](https://api.minebox.co/docs).

| Commande                     | Description                                                                 |
| ---------------------------- | --------------------------------------------------------------------------- |
| `/ping`                      | Répond « Pong ! » avec la latence.                                          |
| `/bestiary <créature>`       | Cherche une créature (autocomplétion) et affiche ses infos et ses **loots avec images**. |
| `/recipes <métier> [search]` | Liste les recettes de craft d'un métier ; détaille les ingrédients et l'item produit. |
| `/market prices <item_id>`   | Statistiques de prix d'un item (achat / vente / direct).                    |
| `/market bazaar <item_id>`   | Offres du bazar pour un item.                                               |
| `/market auction`            | Annonces de l'hôtel des ventes.                                            |
| `/player <pseudo>`           | Profil d'un joueur : niveau, temps de jeu, métiers (niveau + % de progression), stats, compagnons. |
| `/guild <nom>`               | Infos d'une guilde : niveau, XP, membres et nombre de connectés.            |

En plus des commandes, le bot peut afficher dans son **statut** (« petite bulle »)
le nombre de membres connectés d'une guilde (variable `GUILD_PRESENCE`), et
maintenir un **tableau de bord live** de stats joueurs dans un salon
(`STATS_CHANNEL_ID`), tous deux rafraîchis chaque minute.

> Note : certaines routes du marché peuvent renvoyer un corps vide côté API
> (marché inactif) ; le bot l'indique alors clairement.

## Architecture

| Fichier                 | Rôle                                                  |
| ----------------------- | ----------------------------------------------------- |
| `src/main.rs`           | Démarrage du bot et enregistrement des commandes.     |
| `src/api.rs`            | Client HTTP typé de l'API Minebox.                    |
| `src/util.rs`           | Helpers (couleurs de rareté, images base64, formats). |
| `src/commands/`         | Une commande par fichier.                             |

## Licence

MIT
