use crate::card::{
    AbilityCost, AbilityEffect, ActivatedAbility, CardDef, CardType, Color, ManaCost, TargetCost,
};

pub fn summon_cards_into_existence() -> Vec<CardDef> {
    vec![CardDef {
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
