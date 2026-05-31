use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use cards::summon_cards_into_existence;

#[path = "./cards.rs"]
mod cards;

#[derive(Default)]
pub struct CardsBuilder {
    pub next_id: u32,
    pub lookup: HashMap<CardId, CardDefId>,
}

impl CardsBuilder {
    pub fn next(&mut self, did: CardDefId) -> Card {
        let id = CardId(self.next_id);
        self.next_id += 1;
        self.lookup.insert(id, did);
        Card(id)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Cards {
    /// Global list of card definitions.
    #[serde(skip)]
    pub card_defs: Vec<CardDef>,

    pub card_def_lookup: HashMap<CardId, CardDefId>,
}

impl Cards {
    pub fn new(builder: CardsBuilder) -> Self {
        Self {
            card_defs: summon_cards_into_existence(),
            card_def_lookup: builder.lookup,
        }
    }

    pub fn reload_card_defs(&mut self) {
        self.card_defs = summon_cards_into_existence();
    }

    pub fn def_for(&self, cid: CardId) -> &CardDef {
        let Some(did) = self.card_def_lookup.get(&cid) else {
            panic!("No card def found for id {cid:?}");
        };

        &self.card_defs[did.0]
    }
}

/// Like the info that appears on the card, the info about the card used in the game.
#[derive(Debug)]
pub struct CardDef {
    pub name: String,
    pub cost: Vec<ManaCost>,
    pub r#type: Vec<CardType>,

    /// Some cards have an additional color indicator, separate from their mana
    /// colors. These are generally cards with no mana cost or only colorless
    /// mana cost.
    pub color_indicator: Vec<Color>,

    pub activated_abilities: ActivatedAbility,
}

/// Just the index of the card definition within the global card definition list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardDefId(pub usize);

/// Unique identifier for a card in play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardId(pub u32);

/// A card as it appears in like the library or the hand.
///
/// Does not implement [`Clone`] so that we can use ownership to model the
/// uniqueness of each card, ensuring we never accidentally duplicate the card.
#[derive(Debug, Serialize, Deserialize)]
pub struct Card(CardId);

impl Card {
    pub fn id(&self) -> CardId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardType {
    Land,
    Creature { subtypes: Vec<String> },
    Sorcery,
    Artifact,
    Enchantment,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ManaCost {
    /// No mana cost, generally means the card can't be played normally.
    ///
    /// CR: 202.1b Some objects have no mana cost, e.g. lands.
    Nonexistent,

    /// A fixed number of mana of a specified color.
    Fixed { color: ColorCost, quantity: u8 },

    /// A variable amount of mana based on a query of the game state.
    Query,

    /// A variable amount of mana based on some user choice.
    Input,
}

/// CR 202.2a The five colors.
/// CR 202.2b Colorless mana.
#[derive(Debug, Serialize, Deserialize)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ColorCost {
    Single(Color),
    Hybrid(Color, Color),
    Phyrexian(Color),
    Snow,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivatedAbility {
    pub cost: Vec<AbilityCost>,
    pub effect: AbilityEffect,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AbilityCost {
    Mana(ManaCost),
    Tap(TargetCost),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TargetCost {
    TargetSelf,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AbilityEffect {
    GetMana { color: Color, quantity: u8 },
}
