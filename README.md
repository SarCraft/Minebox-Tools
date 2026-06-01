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

| Commande | Description                         |
| -------- | ----------------------------------- |
| `/ping`  | Répond « Pong ! » avec la latence.  |

## Licence

MIT
