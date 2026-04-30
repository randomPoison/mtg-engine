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

/// A card as it appears in like the library or the hand. The number is just the
/// idex in the global card def list, ez
pub struct Card(pub usize);

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

#[derive(Debug)]
pub struct ActivatedAbility {
    pub cost: Vec<AbilityCost>,
    pub effect: AbilityEffect,
}

#[derive(Debug)]
pub enum AbilityCost {
    Mana(ManaCost),
    Tap(TargetCost),
}

#[derive(Debug)]
pub enum TargetCost {
    TargetSelf,
}

#[derive(Debug)]
pub enum AbilityEffect {
    GetMana { color: Color, quantity: u8 },
}
