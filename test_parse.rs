use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaseCard {
    pub id: String,
    pub name: String,
    pub rarity: String,
    #[serde(default)]
    pub pack: String,
    #[serde(rename = "type")]
    #[serde(default)]
    pub card_type: String,
    #[serde(default)]
    pub image: String,
}

#[tokio::main]
async fn main() {
    let url = "https://raw.githubusercontent.com/chase-mew/pokemon-tcg-pocket-cards/main/v1.json";
    let resp = reqwest::get(url).await.unwrap();
    let text = resp.text().await.unwrap();
    match serde_json::from_str::<Vec<ChaseCard>>(&text) {
        Ok(cards) => println!("Success! {} cards", cards.len()),
        Err(e) => println!("Failed to parse: {}", e),
    }
}
