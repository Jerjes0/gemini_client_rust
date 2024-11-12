pub mod gemini_client {
    use reqwest::Client;
    use serde::Serialize;
    use serde_json::{self, Value};
    use std::env;

    #[derive(Serialize)]
    struct ContentPart {
        text: String,
    }

    #[derive(Serialize)]
    struct Content {
        parts: Vec<ContentPart>,
    }

    #[derive(Serialize)]
    struct GenerationConfig {
        response_mime_type: String,
        response_schema: ResponseSchema,
    }

    #[derive(Serialize)]
    struct ResponseSchema {
        r#type: String,
        items: ResponseSchemaItem,
    }

    #[derive(Serialize)]
    struct ResponseSchemaItem {
        r#type: String,
        properties: ResponseProperties,
    }

    #[derive(Serialize)]
    struct ResponseProperties {
        keyword: ResponseType,
    }

    #[derive(Serialize)]
    struct ResponseType {
        r#type: String,
    }

    #[derive(Serialize)]
    struct RequestBody {
        contents: Vec<Content>,
        generation_config: GenerationConfig,
    }

    pub async fn generate_related_words(
        inner_prompt: &str,
        inner_list: &[String],
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let api_key = env::var("GEMINI_API_KEY")?;
        let client = Client::new();

        let list_str = inner_list.join(", ");
        let refined_prompt = format!(
            "Which words from this list [{}] is most closely related to this prompt '{}', Just return the three most related words, nothing else. Make sure that the returned words exist in the list I provided",
            list_str, inner_prompt
        );

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            api_key
        );

        let request_body = RequestBody {
            contents: vec![Content {
                parts: vec![ContentPart {
                    text: refined_prompt,
                }],
            }],
            generation_config: GenerationConfig {
                response_mime_type: "application/json".to_string(),
                response_schema: ResponseSchema {
                    r#type: "ARRAY".to_string(),
                    items: ResponseSchemaItem {
                        r#type: "OBJECT".to_string(),
                        properties: ResponseProperties {
                            keyword: ResponseType {
                                r#type: "STRING".to_string(),
                            },
                        },
                    },
                },
            },
        };

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            let parsed_json: Value = serde_json::from_str(&response_text)?;

            if let Some(text_field) = parsed_json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                let related_words: Vec<Value> = serde_json::from_str(text_field)?;
                let word_list: Vec<String> = related_words.iter().filter_map(|entry| {
                    entry["keyword"].as_str().map(|s| s.to_string())
                }).collect();

                Ok(word_list)
            } else {
                Err("Failed to extract the text field from the response.".into())
            }
        } else {
            Err(format!("Request failed with status: {}", response.status()).into())
        }
    }
}
