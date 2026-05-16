pub use cards::summon_cards_into_existence;
use serde::{Deserialize, Serialize};

#[path = "./cards.rs"]
mod cards;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardId(pub u32);

/// A card as it appears in like the library or the hand.
///
/// Does not implement [`Clone`] so that we can use ownership to model the
/// uniqueness of each card, ensuring we never accidentally duplicate the card.
#[derive(Debug, Serialize, Deserialize)]
pub struct Card(CardId, CardDefId);

impl Card {
    pub fn id(&self) -> CardId {
        self.0
    }

    pub fn def(&self) -> CardDefId {
        self.1
    }
}

/// Generator for [`Card`] objects, ensuring that each card gets a unique ID.
///
/// Currently nothing prevents us from using multiple `CardGen`s to generate
/// multiple [`Card`] objects with the same ID. So be sure to only make one
/// `CardGen` per game state!
pub struct CardGen(u32);

impl CardGen {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn next(&mut self, def: CardDefId) -> Card {
        let id = self.0;
        self.0 += 1;
        Card(CardId(id), def)
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
