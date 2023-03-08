use std::error::Error;

use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};

pub async fn reply(prompt: String) -> Result<Vec<String>, Box<dyn Error>> {
    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo")
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content("You are a helpful assistant.")
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(prompt)
                .build()?,
        ])
        .build()?;

    let response = client.chat().create(request).await?;

    Ok(response
        .choices
        .into_iter()
        .map(|choice| choice.message.content)
        .collect())
}
