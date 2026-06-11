use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub struct OfficialCard {
    pub set: String,
    pub number: u32,
    pub name: String,
    pub rarity: String,

    #[serde(default)]
    pub packs: Vec<String>,

    #[serde(rename="type", default)]
    pub card_type: String,

    // Generated dynamically in main.rs
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

    pub fn trade_card(&mut self, card: &Card, from_account: &str, to_account: &str, quantity: i32) -> Result<(), String> {
        self.remove_card(card,from_account, quantity)?;
        self.add_card(card.clone(), to_account, quantity);

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
}
