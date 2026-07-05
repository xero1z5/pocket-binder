use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generate a resized/optimized image URL via the wsrv.nl proxy.
/// `width` controls the output pixel width (e.g. 400 for grid thumbnails, 600 for detail views).
pub fn optimized_image_url(original_url: &str, _width: u16) -> String {
    original_url.to_string()
}

// this represents the entire database file state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardCollection {
    pub accounts: Vec<Account>,
    pub inventory: Vec<Inventory>,
    #[serde(default)]
    pub wishlist: Vec<Card>,
    #[serde(default)]
    pub tradable: Vec<String>,
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
    pub name: String,
    pub rarity: String, 
    pub card_type: String,
    pub pack: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Inventory {
    pub card: Card,
    pub owners: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackSet {
    pub code: String,
    #[serde(rename = "releaseDate")]
    #[serde(default)]
    pub release_date: String,
    #[serde(default)]
    pub name: HashMap<String, String>,
    #[serde(default)]
    pub packs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OfficialCard {
    pub set: String,
    pub number: u32,
    pub name: String,
    pub rarity: String,

    #[serde(default)]
    pub packs: Vec<String>,

    #[serde(default)]
    pub image: String,

    // Generated dynamically in main.rs
    #[serde(default)]
    pub card_type: String,
    #[serde(default)]
    pub full_image_url: String,
    #[serde(default)]
    pub generated_id: String,
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

    pub fn trade_card(&mut self, card_giving: &Card, card_taking: &Card, my_acc: &str, partner_acc: &str) -> Result<(), String> {
        if !self.inventory.iter().any(|e| &e.card == card_giving && e.owners.get(my_acc).copied().unwrap_or(0) > 0) {
            return Err(format!("{} does not own {}!", my_acc, card_giving.name));
        }

        if partner_acc != "Other" {
            if !self.inventory.iter().any(|e| &e.card == card_taking && e.owners.get(partner_acc).copied().unwrap_or(0) > 0) {
                return Err(format!("{} does not own {}!", partner_acc, card_taking.name));
            }
            
            self.remove_card(card_taking, partner_acc, 1)?;
            self.add_card(card_giving.clone(), partner_acc, 1);
        }

        self.remove_card(card_giving, my_acc, 1)?;
        self.add_card(card_taking.clone(), my_acc, 1);

        Ok(())
    }

    pub fn remove_account(&mut self, account_name: &str){
        self.accounts.retain(|a| a.name!=account_name);

        for entry in &mut self.inventory {
            entry.owners.remove(account_name);
        }

        self.inventory.retain(|entry| !entry.owners.is_empty());
    }

    pub fn set_account_main_status(&mut self, account_name: &str, is_main: bool){
        if let Some(acc) = self.accounts.iter_mut().find(|a| a.name==account_name){
            acc.main=is_main;
        }
    }

    pub fn update_account(&mut self, old_name: &str, new_name: &str, new_id: &str, new_main: bool) {
        if let Some(acc) = self.accounts.iter_mut().find(|a| a.name == old_name) {
            acc.name = new_name.to_string();
            acc.id = new_id.to_string();
            acc.main = new_main;
        }

        // Rename the key in every inventory entry's owners map
        if old_name != new_name {
            for entry in &mut self.inventory {
                if let Some(count) = entry.owners.remove(old_name) {
                    entry.owners.insert(new_name.to_string(), count);
                }
            }
        }
    }

    pub fn toggle_wishlist(&mut self, card: Card) {
        if let Some(pos) = self.wishlist.iter().position(|c| c.id == card.id) {
            self.wishlist.remove(pos);
        } else {
            self.wishlist.push(card);
        }
    }

    pub fn is_wishlisted(&self, card_id: &str) -> bool {
        self.wishlist.iter().any(|c| c.id == card_id)
    }

    pub fn toggle_tradable(&mut self, card_id: &str) {
        if let Some(pos) = self.tradable.iter().position(|id| id == card_id) {
            self.tradable.remove(pos);
        } else {
            self.tradable.push(card_id.to_string());
        }
    }

    pub fn is_tradable(&self, card_id: &str) -> bool {
        self.tradable.contains(&card_id.to_string())
    }

    /// Automatically repairs orphaned cards (e.g., from old database IDs like TCGdex) 
    /// by matching their name and rarity against the new image_db.
    pub fn migrate_orphaned_cards(&mut self, api_map: &HashMap<String, OfficialCard>) -> bool {
        let mut modified = false;

        // Helper closure to fix a single card
        let fix_card = |card: &mut Card, api_map: &HashMap<String, OfficialCard>| -> bool {
            if !api_map.contains_key(&card.id) {
                // Try to find a match by name and rarity
                if let Some(new_card) = api_map.values().find(|c| c.name == card.name && c.rarity == card.rarity) {
                    card.id = new_card.generated_id.clone();
                    card.pack = new_card.set.clone();
                    if card.card_type.is_empty() {
                        card.card_type = new_card.card_type.clone();
                    }
                    return true;
                }
            }
            false
        };

        // Migrate inventory
        for entry in &mut self.inventory {
            if fix_card(&mut entry.card, api_map) {
                modified = true;
            }
        }

        // Migrate wishlist
        for card in &mut self.wishlist {
            if fix_card(card, api_map) {
                modified = true;
            }
        }
        
        // Migrate tradable (this is an array of IDs, so we need to find the inventory entry to get the name/rarity, or just wipe orphaned tradables if they can't be mapped easily)
        // Since we already migrated inventory, we should map tradables based on the NEW inventory IDs if they changed.
        // Actually, tradable just holds IDs. It's safer to just let the user re-mark them if they get orphaned, 
        // OR we can map them if we had a mapping dictionary. Since we mutate the inventory in place, 
        // we can just check if the tradable ID exists in inventory. If not, maybe it was migrated to a new ID?
        // To be safe and simple, we'll just clean up tradable IDs that no longer exist in inventory.
        let mut valid_tradables = Vec::new();
        for id in &self.tradable {
            if self.inventory.iter().any(|e| &e.card.id == id) {
                valid_tradables.push(id.clone());
            } else {
                // Try to see if this ID was an old ID that got migrated.
                // We'd have to find it in the old state. Since we didn't track old->new maps,
                // we'll just drop invalid tradables. The user can click the trade button again.
                modified = true;
            }
        }
        self.tradable = valid_tradables;

        modified
    }
}

//=================== TCGDEX API RESPONSE MODELS =======================

/// TCGdex: /v2/en/series/tcgp → .sets[]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcgdexSetBrief {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub logo: Option<String>,
}

/// TCGdex: /v2/en/sets/{id} → full set with cards and boosters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcgdexSetDetail {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub cards: Vec<TcgdexCardBrief>,
    #[serde(default)]
    pub boosters: Vec<TcgdexBooster>,
}

/// TCGdex: card entry inside a set response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcgdexCardBrief {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(rename = "localId")]
    #[serde(default)]
    pub local_id: String,
}

/// TCGdex: booster/pack entry inside a set response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcgdexBooster {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
}

/// TCGdex: /v2/en/series/tcgp response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcgdexSeriesResponse {
    #[serde(default)]
    pub sets: Vec<TcgdexSetBrief>,
}

/// Flibustier npm CDN: PRIMARY card data source
/// Fetched from: https://unpkg.com/pokemon-tcg-pocket-database@latest/dist/cards.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlibustierCard {
    #[serde(default)]
    pub set: String,
    #[serde(default)]
    pub number: u32,
    #[serde(default)]
    pub rarity: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub packs: Vec<String>,
}

/// Flibustier npm CDN: set data
/// Fetched from: https://unpkg.com/pokemon-tcg-pocket-database@latest/dist/sets.json
/// Response is a HashMap<String, Vec<FlibustierSet>> keyed by series letter (A, B, PROMO)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlibustierSet {
    pub code: String,
    #[serde(rename = "releaseDate")]
    #[serde(default)]
    pub release_date: String,
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub name: HashMap<String, String>,
    #[serde(default)]
    pub packs: Vec<String>,
}

