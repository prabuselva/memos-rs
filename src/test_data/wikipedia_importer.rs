use crate::models::Note;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct WikipediaSearchResponse {
    query: WikipediaQuery,
}

#[derive(Debug, Deserialize)]
struct WikipediaQuery {
    search: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[allow(private_interfaces)]
struct SearchResult {
    title: String,
    #[allow(dead_code)]
    snippet: String,
}

pub async fn search_wikipedia(
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, anyhow::Error> {
    let client = Client::new();
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&srlimit={}&format=json",
        urlencoding::encode(query),
        limit
    );

    let response = client.get(&url).send().await?;
    let json: WikipediaSearchResponse = response.json().await?;

    Ok(json.query.search)
}

pub async fn get_wikipedia_page(title: &str) -> Result<String, anyhow::Error> {
    let client = Client::new();
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&prop=extracts&exintro=&explaintext=&titles={}&format=json",
        urlencoding::encode(title)
    );

    let response = client.get(&url).send().await?;
    let json: serde_json::Value = response.json().await?;

    if let Some(pages) = json.get("query").and_then(|q| q.get("pages")) {
        if let Some(first_page) = pages.as_object().and_then(|p| p.values().next()) {
            if let Some(extract) = first_page.get("extract").and_then(|e| e.as_str()) {
                return Ok(extract.to_string());
            }
        }
    }

    Err(anyhow::anyhow!("No page found"))
}

pub async fn import_wikipedia_notes(
    category: &str,
    count: usize,
    user_id: &str,
) -> Result<Vec<Note>, anyhow::Error> {
    let search_results = search_wikipedia(category, count * 2).await?;

    let mut notes = Vec::new();
    for result in search_results.iter().take(count) {
        let content = get_wikipedia_page(&result.title).await?;

        let note = Note::new(result.title.clone(), content)
            .with_user_id(user_id.to_string())
            .with_tags(vec![category.to_string(), "wikipedia".to_string()]);

        notes.push(note);
    }

    Ok(notes)
}
