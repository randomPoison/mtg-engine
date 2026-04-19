use crate::card::{
    AbilityCost, AbilityEffect, ActivatedAbility, Card, CardType, Color, ManaCost, TargetCost,
};

pub fn cards() -> Vec<Card> {
    vec![Card {
        name: "Mountain".into(),
        cost: vec![ManaCost::Nonexistent],
        r#type: vec![CardType::Land],
        color_indicator: vec![],
        activated_abilities: ActivatedAbility {
            cost: vec![AbilityCost::Tap(TargetCost::TargetSelf)],
            effect: AbilityEffect::GetMana {
                color: Color::Red,
                quantity: 1,
            },
        },
    }]
}
