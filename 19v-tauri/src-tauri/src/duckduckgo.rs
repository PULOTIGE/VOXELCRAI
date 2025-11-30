use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::{Client, Url};
use serde::Deserialize;

use crate::simulation::{ConceptOrigin, SharedSimulation};

const KEYWORDS: &[&str] = &[
    "19V",
    "Alushta",
    "nttrl",
    "KSB",
    "CrimeaAI",
    "хаос Лоренца",
    "ДНК-кодирование",
];

#[derive(Debug, Deserialize)]
struct DuckDuckGoResponse {
    #[serde(default)]
    AbstractText: String,
    #[serde(default)]
    Heading: String,
    #[serde(default)]
    RelatedTopics: Vec<RelatedTopic>,
}

#[derive(Debug, Deserialize)]
struct RelatedTopic {
    #[serde(default)]
    Text: String,
    #[serde(default)]
    FirstURL: String,
    #[serde(default)]
    Topics: Vec<RelatedTopic>,
}

pub fn spawn_ingestion(shared: SharedSimulation) {
    let client = Client::builder()
        .user_agent("19V-Autonomous/3.0")
        .build()
        .expect("reqwest client");

    tauri::async_runtime::spawn(async move {
        loop {
            if let Err(err) = ingest_cycle(&client, &shared).await {
                shared.log(
                    crate::simulation::LogLevel::Warn,
                    format!("DuckDuckGo ingest error: {err}"),
                );
            }
            tauri::async_runtime::sleep(Duration::from_secs(60 * 19)).await;
        }
    });
}

async fn ingest_cycle(client: &Client, shared: &SharedSimulation) -> Result<()> {
    let query = KEYWORDS.join(" OR ");
    let url = Url::parse_with_params(
        "https://api.duckduckgo.com/",
        [
            ("q", query.as_str()),
            ("format", "json"),
            ("no_redirect", "1"),
            ("no_html", "1"),
        ],
    )?;

    let response = client
        .get(url)
        .send()
        .await
        .context("DuckDuckGo request failed")?
        .error_for_status()?;

    let payload: DuckDuckGoResponse = response.json().await?;
    let mut concepts = Vec::new();
    if !payload.AbstractText.is_empty() {
        concepts.push(payload.AbstractText.clone());
    }
    if !payload.Heading.is_empty() {
        concepts.push(payload.Heading.clone());
    }
    collect_topics(&payload.RelatedTopics, &mut concepts);

    if concepts.is_empty() {
        shared.log(
            crate::simulation::LogLevel::Info,
            "DuckDuckGo не вернул концепты",
        );
        return Ok(());
    }

    for concept in concepts.into_iter().take(12) {
        shared.ingest_concepts(&concept, ConceptOrigin::DuckDuckGo, concept.as_bytes());
    }

    shared.log(
        crate::simulation::LogLevel::Info,
        "Организм поглотил свежие концепты DuckDuckGo",
    );
    Ok(())
}

fn collect_topics(topics: &[RelatedTopic], buffer: &mut Vec<String>) {
    for topic in topics {
        if !topic.Text.is_empty() {
            buffer.push(topic.Text.clone());
        }
        if !topic.FirstURL.is_empty() {
            buffer.push(topic.FirstURL.clone());
        }
        if !topic.Topics.is_empty() {
            collect_topics(&topic.Topics, buffer);
        }
    }
}
