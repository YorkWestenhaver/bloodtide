use bevy::prelude::*;
use rand::Rng;

/// Type of card in the deck
#[derive(Debug, Clone, PartialEq)]
pub enum CardType {
    Creature,
    Weapon,
    Artifact,
}

/// A single card in the player's deck
#[derive(Debug, Clone)]
pub struct DeckCard {
    pub card_type: CardType,
    pub id: String,
    pub weight: f64,
}

impl DeckCard {
    pub fn creature(id: &str, weight: f64) -> Self {
        Self {
            card_type: CardType::Creature,
            id: id.to_string(),
            weight,
        }
    }

    pub fn weapon(id: &str, weight: f64) -> Self {
        Self {
            card_type: CardType::Weapon,
            id: id.to_string(),
            weight,
        }
    }

    pub fn artifact(id: &str, weight: f64) -> Self {
        Self {
            card_type: CardType::Artifact,
            id: id.to_string(),
            weight,
        }
    }
}

/// The player's deck of cards, used for rolling rewards on level up
#[derive(Resource)]
pub struct PlayerDeck {
    pub cards: Vec<DeckCard>,
    pub total_weight: f64,
}

impl PlayerDeck {
    pub fn new(cards: Vec<DeckCard>) -> Self {
        let total_weight = cards.iter().map(|c| c.weight).sum();
        Self {
            cards,
            total_weight,
        }
    }

    /// Roll a random card from the deck using weighted selection
    pub fn roll_card(&self) -> Option<&DeckCard> {
        if self.cards.is_empty() || self.total_weight <= 0.0 {
            return None;
        }

        let mut rng = rand::thread_rng();
        let roll: f64 = rng.gen_range(0.0..self.total_weight);

        let mut cumulative = 0.0;
        for card in &self.cards {
            cumulative += card.weight;
            if roll < cumulative {
                return Some(card);
            }
        }

        // Fallback to last card (shouldn't happen with correct weights)
        self.cards.last()
    }
}

impl Default for PlayerDeck {
    fn default() -> Self {
        Self::new(vec![])
    }
}
