#[derive(Debug)]
pub struct Card {
    pub name: String,
    pub cost: Vec<ManaCost>,
    pub r#type: Vec<CardType>,

    /// Some cards have an additional color indicator, separate from their mana
    /// colors. These are generally cards with no mana cost or only colorless
    /// mana cost.
    pub color_indicator: Vec<Color>,
}

#[derive(Debug)]
pub enum CardType {
    Land,
    Creature { subtypes: Vec<String> },
    Sorcery,
    Artifact,
    Enchantment,
}

#[derive(Debug)]
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
#[derive(Debug)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

#[derive(Debug)]
pub enum ColorCost {
    Single(Color),
    Hybrid(Color, Color),
    Phyrexian(Color),
    Snow,
}
