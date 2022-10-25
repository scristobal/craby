# Craby

Craby is a Telegram bot  written in Rust 🤖🦀 that runs ML models hosted in replicate.com 

Currently, only Stable Diffusion v1.4 is supported. 

Give it a try https://t.me/test_botatoide_bot it will answer to `/make a horse in space` 

## Environment

Requires `TELOXIDE_TOKEN`, `R8_TOKEN` and `PUBLIC_URL` set on the environment or a `.env` file.

- `TELOXIDE_TOKEN` is the Telegram bot token, obtained from [here](https://core.telegram.org/bots#6-botfather).

- `R8_TOKEN` is your private replicate.com token [API ref](https://replicate.ai/docs/api/).

- `PUBLIC_URL` is the URL of the server where the bot is running. eg. `http://example.com/`

## Debug

Run with `RUST_LOG=debug cargo run`

## Roadmap

- [ ] Support for multiple models
- [ ] Improve error handling, maybe use `thiserror`?
