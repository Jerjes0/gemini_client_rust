use gloo_net::http::Request;
use serde::Serialize;
use serde_json::{self, Value};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_utils::format::JsValueSerdeExt;
use web_sys::console;

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

#[wasm_bindgen]
pub async fn generate_related_words(api_key: String, inner_prompt: String, inner_list: Vec<String>) -> Result<JsValue, JsValue> {
    // Use the passed API key directly
    let list_str = inner_list.join(", ");
    let refined_prompt = format!(
        "Which words from this list [{}] is most closely related to this prompt '{}', Just return the three most related words, nothing else. Make sure that the returned words exist in the list I provided",
        list_str, inner_prompt
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

    let body_json = serde_json::to_string(&request_body).map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))?;

    let response = Request::post(&format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
        api_key
    ))
    .header("Content-Type", "application/json")
    .body(body_json)
    .map_err(|e| JsValue::from_str(&format!("Failed to build request body: {}", e)))?
    .send()
    .await
    .map_err(|e| JsValue::from_str(&format!("Failed to send request: {}", e)))?;

    if response.ok() {
        let response_text = response.text().await.map_err(|e| JsValue::from_str(&format!("Failed to read response: {}", e)))?;
        let parsed_json: Value = serde_json::from_str(&response_text).map_err(|e| JsValue::from_str(&format!("Failed to parse response: {}", e)))?;

        if let Some(text_field) = parsed_json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            let related_words: Vec<Value> = serde_json::from_str(text_field).map_err(|e| JsValue::from_str(&format!("Failed to deserialize response: {}", e)))?;
            let word_list: Vec<String> = related_words.iter().filter_map(|entry| {
                entry["keyword"].as_str().map(|s| s.to_string())
            }).collect();

            Ok(JsValue::from_serde(&word_list).map_err(|e| JsValue::from_str(&format!("Failed to convert to JsValue: {}", e)))?)
        } else {
            Err(JsValue::from_str("Failed to extract the text field from the response"))
        }
    } else {
        Err(JsValue::from_str(&format!("Request failed with status: {}", response.status())))
    }
}

// Helper function to run async tasks in WASM
#[wasm_bindgen]
pub fn run_async_task(api_key: String, inner_prompt: String, inner_list: Vec<String>) {
    spawn_local(async move {
        match generate_related_words(api_key, inner_prompt, inner_list).await {
            Ok(result) => {
                console::log_1(&result);
            }
            Err(e) => {
                console::error_1(&e);
            }
        }
    });
}