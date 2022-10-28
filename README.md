# Craby

Craby is a Telegram bot  written in Rust 🤖🦀 that runs ML models hosted in replicate.com 

Currently, [Stable Diffusion v1.4](https://replicate.com/stability-ai/stable-diffusion) and [Dalee-mini](https://replicate.com/kuprel/min-dalle) are supported, both with text input only - it is not possible to set a seed or other parameters.

Give it a try https://t.me/test_botatoide_bot The bot will answer to `/stabled a horse in space` or `/dallem a horse in space`

## Environment

Requires `TELOXIDE_TOKEN`, `R8_TOKEN` and `PUBLIC_URL` set on the environment or a `.env` file.

- `TELOXIDE_TOKEN` is the Telegram bot token, obtained from [here](https://core.telegram.org/bots#6-botfather).
- `R8_TOKEN` is your private replicate.com token [API ref](https://replicate.ai/docs/api/).
- `PUBLIC_URL` is the URL of the server where the bot is running. eg. `http://example.com/`

## Debug

Use something like `ngrok` to get a public url, run with `RUST_LOG=debug cargo run` 

## Notes

The public url is required for the webhook server, in the future a pooling mechanism can be used instead. 

## Roadmap

- [x] Support for multiple models
- [x] Improve error handling, maybe use `thiserror`?
- [ ] Minimal website and donations
- [ ] Suport image inputs, eg. real-esrgan
- [ ] Support text to text, eg. prompt-parrot 
- [ ] Support sound input, eg. openai/whisper
- [ ] (maybe) Pooling mechanism if `PUBLIC_URL` is not set. 
