use dioxus::{html::{a::target, script::r#async}, prelude::*};
use core::sync;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use base64::Engine;

//=================== APP ====================

// this represents the entire database file state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardCollection {
    pub accounts: Vec<Account>,
    pub inventory: Vec<Inventory>,
}

// account definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub name: String,
    pub id: String,
    pub main: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Card {
    pub id: String,
    pub name: String, // card name
    pub rarity: String, // ? enum -> shiny fa, immersive, 2* (RR, FA, Trainers)
    pub card_type: String, // pokemon, trainer
    pub pack: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Inventory {
    pub card: Card,
    pub owners: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OfficialCard {
    pub id: String,
    pub name: String,
    pub image: String,
    pub pack: String,
    pub rarity: String,

    #[serde(rename="type", default)]
    pub card_type: String,
}

//=================== LOGIC =======================

impl CardCollection {
     
    pub fn add_card(&mut self, card: Card, account_name: &str, quantity: i32){

        if let Some(entry)=self.inventory.iter_mut().find(|e| e.card==card){
            let current_count = entry.owners.entry(account_name.to_string()).or_insert(0);
            *current_count+=quantity;
        }
        else {
            let mut new_owners = HashMap::new();
            new_owners.insert(account_name.to_string(),quantity);

            self.inventory.push(Inventory{
                card,
                owners: new_owners,
            });
        }
    }

    pub fn remove_card(&mut self, card: &Card, account_name: &str, quantity: i32) -> Result<(),String> {
        let entry_index=self.inventory.iter().position(|e| &e.card==card);

        if let Some(index)=entry_index{
            let entry = &mut self.inventory[index];

            if let Some(current_count) = entry.owners.get_mut(account_name){
                if *current_count >= quantity{
                    *current_count-=quantity;

                    if *current_count==0{
                        entry.owners.remove(account_name);
                    }
                }
                else{
                    return Err(format!("{} does not have enough of this card.", account_name));
                }
            }
            else{
                return Err(format!("{} does not own this card.", account_name));
            }

            if self.inventory[index].owners.is_empty() {
                self.inventory.remove(index);
            }
            Ok(())
        }
        else {
            Err("Card not found in database".to_string())
        }
    }

    pub fn trade_card(&mut self, card: &Card, from_account: &str, to_account: &str, quantity: i32) -> Result<(), String> {
        self.remove_card(card,from_account, quantity)?;
        self.add_card(card.clone(), to_account, quantity);

        Ok(())
    }
}

//======================= DX ===================

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // =========================================================================
    // STATE SIGNALS
    let mut github_username = use_signal(|| String::new());
    let mut github_token = use_signal(|| String::new());

    let mut selected_account_filter = use_signal(|| String::from("All"));
    let mut search_query = use_signal(|| String::new());
    let mut collection = use_signal(generate_mock_data);

    // State for the Add Card Modal
    let mut show_add_modal = use_signal(|| false);
    let mut add_search_query = use_signal(|| String::new());
    let mut add_target_account = use_signal(|| String::from("kurapika"));

    // state for syncing
    let mut sync_status = use_signal(|| String::from(""));


    // =========================================================================
    // 2. FETCH OFFICIAL API DATABASE
    // =========================================================================
    let image_db = use_resource(move || async move {
        let url = "https://raw.githubusercontent.com/chase-manning/pokemon-tcg-pocket-cards/refs/heads/main/v4.json";

        let response = reqwest::get(url).await.ok()?;
        let official_cards = response.json::<Vec<OfficialCard>>().await.ok()?;

        let mut api_db: HashMap<String, OfficialCard> = HashMap::new();
        for card in official_cards {
            api_db.insert(card.id.clone(), card);
        }
        
        Some(api_db)
    });

    // =========================================================================
    // 3. UI LAYOUT
    // =========================================================================
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } 
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "bg-gray-900 text-white min-h-screen p-6 font-sans",
            
            // --- HEADER ---
            header { class: "border-b border-gray-800 pb-4 mb-6 flex flex-col md:flex-row justify-between items-start md:items-center gap-4",
                h1 { class: "text-2xl font-black tracking-tight text-transparent bg-clip-text bg-gradient-to-r from-red-500 to-orange-400", 
                    "🧬 POCKET TCG BINDER" 
                }
                
                // Dual Input Login Section
                div { class: "flex flex-col md:flex-row items-center gap-2 w-full md:w-auto",
                    input {
                        r#type: "text",
                        class: "bg-gray-800 border border-gray-700 text-sm rounded px-3 py-1.5 focus:outline-none focus:border-orange-500 text-gray-300 w-full md:w-48",
                        placeholder: "GitHub Username",
                        value: "{github_username}",
                        oninput: move |evt| github_username.set(evt.value())
                    }
                    input {
                        r#type: "password",
                        class: "bg-gray-800 border border-gray-700 text-sm rounded px-3 py-1.5 focus:outline-none focus:border-orange-500 text-gray-300 w-full md:w-64",
                        placeholder: "ghp_xxxxxxxxxxxx (Token)",
                        value: "{github_token}",
                        oninput: move |evt| github_token.set(evt.value())
                    }
                }
            }

            // --- FILTER BAR ---
            div { class: "flex flex-col md:flex-row gap-4 mb-6",
                // Search Input
                input {
                    class: "flex-1 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 focus:outline-none focus:border-blue-500",
                    placeholder: "Search by card name...",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value())
                }

                // Account Filter Dropdown
                select {
                    class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 focus:outline-none focus:border-blue-500 cursor-pointer",
                    onchange: move |evt| selected_account_filter.set(evt.value()),
                    option { value: "All", "All Accounts" }
                    for account in collection.read().accounts.iter() {
                        option { value: "{account.name}", "{account.name}" }
                    }
                }

                // New Card Button
                button {
                    class: "bg-orange-600 hover:bg-orange-500 text-white font-bold py-2 px-6 rounded-lg shadow-lg transition-transform active:scale-95",
                    onclick: move |_| show_add_modal.set(true),
                    "➕ Add Card"
                }

                // save to GtiHub Button
                button {
                    class: "bg-green-600 hover:bg-green-500 text-white font-bold py-2 px-6 rounded-lg shadow-lg transition-transform active:scale-95 flex items-center gap-2",
                    onclick: move |_| {
                        sync_status.set("⏳ Saving...".to_string());

                        // capture all current states
                        let current_collection = collection.read().clone();
                        let user_to_use = github_username.read().clone();
                        let token_to_use = github_token.read().clone();

                        // spawn the async background task
                        spawn(async move {
                            match save_to_github(current_collection, user_to_use, token_to_use).await {
                                Ok(_) => sync_status.set("✅ Saved!".to_string()),
                                Err(e) => sync_status.set(format!("❌ {}",e)),
                            }
                        });
                    },
                    "💾 Save to GitHub",
                }

                // status indicator
                if !sync_status.read().is_empty() {
                    span { class: "text-sm font-mono flex items-center", "{sync_status}" }
                }
            }

            // --- THE VISUAL GRID ---
            div { class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4",
                for entry in collection.read().inventory.iter().filter(|e| {
                    let matches_search = search_query.read().is_empty() || 
                                         e.card.name.to_lowercase().contains(&search_query.read().to_lowercase());
                    
                    let matches_account = &*selected_account_filter.read() == "All" || 
                                          e.owners.contains_key(&*selected_account_filter.read());
                    
                    matches_search && matches_account
                }) {
                    div { class: "bg-gray-800 border border-gray-700 rounded-xl p-3 flex flex-col items-center shadow-lg transition-transform hover:scale-105",

                        // Dynamic Image Rendering
                        {
                            let image_url = if let Some(Some(api_map)) = &*image_db.read() {
                                api_map.get(&entry.card.id).map(|c| c.image.clone())
                            } else {
                                None
                            };

                            if let Some(url) = image_url {
                                rsx! {
                                    img {
                                        src: "{url}",
                                        alt: "{entry.card.name}",
                                        class: "w-full rounded-lg mb-3 shadow-md border border-gray-600 aspect-[63/88] object-cover"
                                    }
                                }
                            } else {
                                rsx!{
                                    div { class: "w-full aspect-[63/88] bg-gray-700 rounded-lg mb-3 flex items-center justify-center border border-gray-600 animate-pulse",
                                        span { class: "text-4xl opacity-50", "🃏" }
                                    }
                                }
                            }
                        }
                        
                        // Card Info
                        h2 { class: "font-bold text-sm text-center truncate w-full", "{entry.card.name}" }
                        p { class: "text-xs text-orange-400 mb-2 uppercase tracking-wide", "{entry.card.rarity}" }
                        
                        // Who owns it?
                        div { class: "w-full flex flex-wrap gap-1 justify-center mt-auto",
                            for (owner_name, count) in entry.owners.iter() {
                                if *count > 0 {
                                    span { class: "text-[10px] bg-blue-900/50 text-blue-200 border border-blue-700/50 px-2 py-0.5 rounded-full font-mono",
                                        "{owner_name}: {count}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            if collection.read().inventory.is_empty() {
                div { class: "text-center text-gray-500 mt-12",
                    "No cards found matching your criteria."
                }
            }
        }
    
        // =========================================================================
        // 4. THE ADD CARD MODAL
        // =========================================================================
        if *show_add_modal.read() {
            div { class: "fixed inset-0 bg-black/90 flex flex-col items-center p-4 md:p-10 z-50",
                
                // Modal Header & Controls
                div { class: "w-full max-w-6xl bg-gray-900 border border-gray-700 rounded-t-xl p-4 flex flex-col md:flex-row gap-4 justify-between items-center shadow-2xl",
                    h2 { class: "text-2xl font-bold text-orange-400", "Add from Database" }
                    
                    div { class: "flex w-full md:w-auto gap-2",
                        input {
                            class: "flex-1 md:w-64 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 focus:outline-none focus:border-blue-500",
                            placeholder: "Search official API...",
                            value: "{add_search_query}",
                            oninput: move |evt| add_search_query.set(evt.value())
                        }
                        
                        select {
                            class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 cursor-pointer",
                            onchange: move |evt| add_target_account.set(evt.value()),
                            for account in collection.read().accounts.iter() {
                                option { value: "{account.name}", "{account.name}" }
                            }
                        }
                        
                        button {
                            class: "bg-red-900/80 hover:bg-red-800 text-red-200 px-4 py-2 rounded-lg font-bold",
                            onclick: move |_| show_add_modal.set(false),
                            "Close"
                        }
                    }
                }

                // Modal Results Grid
                div { class: "w-full max-w-6xl flex-1 bg-gray-800/50 border-x border-b border-gray-700 rounded-b-xl p-4 overflow-y-auto",
                    div { class: "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 gap-3",
                        
                        {
                            // 1. Safely extract and filter the data into an owned Vector FIRST
                            let mut display_cards = Vec::new();
                            
                            if let Some(Some(db)) = &*image_db.read() {
                                let search_term = add_search_query.read().to_lowercase();
                                let mut count = 0;
                                
                                for card in db.values() {
                                    if search_term.is_empty() || card.name.to_lowercase().contains(&search_term) {
                                        display_cards.push(card.clone());
                                        count += 1;

                                        if count>=32{
                                            break;
                                        }
                                    }
                                }
                            }

                            // 2. Now that the `image_db.read()` lock is dropped, we can safely render the UI
                            if display_cards.is_empty() {
                                rsx! { p { class: "col-span-full text-center text-gray-500 mt-10 animate-pulse", "Loading API Database or No Results Found..." } }
                            } else {
                                rsx! {
                                    for official_card in display_cards {
                                        div { 
                                            class: "relative group cursor-pointer transition-transform hover:scale-105 hover:z-10",
                                            
                                            // THE ADD CARD ACTION
                                            onclick: move |_| {
                                                let new_card = Card {
                                                    id: official_card.id.clone(),
                                                    name: official_card.name.clone(),
                                                    rarity: official_card.rarity.clone(),
                                                    card_type: official_card.card_type.clone(),
                                                    pack: official_card.pack.clone(),
                                                };
                                                collection.write().add_card(
                                                    new_card, 
                                                    &*add_target_account.read(), 
                                                    1
                                                );
                                            },
                                            
                                            img { 
                                                src: "{official_card.image}", 
                                                loading: "lazy",
                                                class: "w-full rounded border border-gray-600 shadow-md" 
                                            }
                                            
                                            // Hover Overlay
                                            div { class: "absolute inset-0 bg-blue-500/50 opacity-0 group-hover:opacity-100 rounded flex items-center justify-center backdrop-blur-sm transition-opacity",
                                                span { class: "bg-gray-900 text-white text-xs font-bold px-2 py-1 rounded-full", "+ Add" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// -------------------------------------------------------------------------
// Helper function to build standard dummy data using your exact struct shape
// -------------------------------------------------------------------------
fn generate_mock_data() -> CardCollection {
    let mut accounts = Vec::new();
    accounts.push(Account { name: "kurapika".to_string(), id: "".to_string(), main: true });
    accounts.push(Account { name: "killua".to_string(), id: "8097159320998426".to_string(), main: true });
    accounts.push(Account { name: "pulsing".to_string(), id: "4557890198590873".to_string(), main: false });

    let mut inventory = Vec::new();


    // Let's add the exact Aerodactyl you found!
    let mut aero_owners = HashMap::new();
    aero_owners.insert("kurapika".to_string(), 1);
    inventory.push(Inventory {
        card: Card {
            id: "a1a-084".to_string(), // The exact API ID
            name: "Aerodactyl ex".to_string(),
            rarity: "☆☆".to_string(),
            pack: "Mew".to_string(),
            card_type: "Fighting".to_string(),
        },
        owners: aero_owners,
    });

    CardCollection { accounts, inventory }
}


// =========================================================================
// 5. GITHUB SYNC LOGIC
// =========================================================================

async fn save_to_github(collection: CardCollection, username: String, token: String) -> Result<(), String> {

    if username.is_empty() || token.is_empty() {
        return Err("Both GitHub Username and Token are required.".to_string());
    }

    let repo = "pocket-binder";
    if token.is_empty() {
        return Err("No GitHub Token provided.".to_string());
    }
    let url = format!("https://api.github.com/repos/{}/{}/contents/collection.json", username, repo);
    let client = reqwest::Client::new();

    // 1. GET the current file to grab its SHA hash
    let mut sha = None;
    let get_res = client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Pocket-Binder-App")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if get_res.status().is_success() {
        let json: serde_json::Value = get_res.json().await.map_err(|e| e.to_string())?;
        if let Some(s) = json["sha"].as_str() {
            sha = Some(s.to_string());
        }
    }

    // 2. Convert our Rust Struct back into a formatted JSON string
    let new_json_string = serde_json::to_string_pretty(&collection).map_err(|e| e.to_string())?;
    
    // 3. Base64 Encode the JSON string
    let encoded_content = base64::engine::general_purpose::STANDARD.encode(new_json_string);

    // 4. Prepare the Commit Payload
    let mut payload = serde_json::json!({
        "message": "Updated binder from Web App 🚀",
        "content": encoded_content,
    });

    if let Some(s) = sha {
        payload["sha"] = serde_json::json!(s);
    }

    // 5. PUT the new file back to GitHub
    let put_res = client.put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Pocket-Binder-App")
        .header("Accept", "application/vnd.github.v3+json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if put_res.status().is_success() {
        Ok(())
    } else {
        Err(format!("GitHub API Error: {}", put_res.status()))
    }
}
