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

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // DeckCard Tests
    // =========================================================================

    #[test]
    fn deck_card_creature_creates_correct_type() {
        let card = DeckCard::creature("fire_imp", 10.0);
        assert_eq!(card.card_type, CardType::Creature);
        assert_eq!(card.id, "fire_imp");
        assert_eq!(card.weight, 10.0);
    }

    #[test]
    fn deck_card_weapon_creates_correct_type() {
        let card = DeckCard::weapon("ember_staff", 5.0);
        assert_eq!(card.card_type, CardType::Weapon);
        assert_eq!(card.id, "ember_staff");
        assert_eq!(card.weight, 5.0);
    }

    #[test]
    fn deck_card_artifact_creates_correct_type() {
        let card = DeckCard::artifact("molten_core", 2.5);
        assert_eq!(card.card_type, CardType::Artifact);
        assert_eq!(card.id, "molten_core");
        assert_eq!(card.weight, 2.5);
    }

    // =========================================================================
    // PlayerDeck Tests
    // =========================================================================

    #[test]
    fn player_deck_calculates_total_weight_correctly() {
        let cards = vec![
            DeckCard::creature("fire_imp", 10.0),
            DeckCard::creature("ember_hound", 8.0),
            DeckCard::weapon("ember_staff", 5.0),
            DeckCard::artifact("molten_core", 2.0),
        ];
        let deck = PlayerDeck::new(cards);
        assert_eq!(deck.total_weight, 25.0);
    }

    #[test]
    fn player_deck_empty_has_zero_weight() {
        let deck = PlayerDeck::new(vec![]);
        assert_eq!(deck.total_weight, 0.0);
        assert!(deck.cards.is_empty());
    }

    #[test]
    fn player_deck_default_is_empty() {
        let deck = PlayerDeck::default();
        assert!(deck.cards.is_empty());
        assert_eq!(deck.total_weight, 0.0);
    }

    #[test]
    fn player_deck_roll_card_returns_none_for_empty_deck() {
        let deck = PlayerDeck::new(vec![]);
        assert!(deck.roll_card().is_none());
    }

    #[test]
    fn player_deck_roll_card_returns_none_for_zero_weight() {
        let cards = vec![
            DeckCard::creature("fire_imp", 0.0),
            DeckCard::creature("ember_hound", 0.0),
        ];
        let deck = PlayerDeck::new(cards);
        assert!(deck.roll_card().is_none());
    }

    #[test]
    fn player_deck_single_card_always_returns_that_card() {
        let cards = vec![DeckCard::creature("fire_imp", 10.0)];
        let deck = PlayerDeck::new(cards);

        // Roll multiple times - should always get the same card
        for _ in 0..10 {
            let rolled = deck.roll_card();
            assert!(rolled.is_some());
            assert_eq!(rolled.unwrap().id, "fire_imp");
        }
    }

    #[test]
    fn player_deck_roll_returns_valid_card_from_deck() {
        let cards = vec![
            DeckCard::creature("fire_imp", 10.0),
            DeckCard::creature("ember_hound", 10.0),
            DeckCard::weapon("ember_staff", 10.0),
        ];
        let deck = PlayerDeck::new(cards);

        // Roll many times - all results should be valid deck cards
        for _ in 0..100 {
            let rolled = deck.roll_card();
            assert!(rolled.is_some());
            let id = &rolled.unwrap().id;
            assert!(
                id == "fire_imp" || id == "ember_hound" || id == "ember_staff",
                "Unexpected card id: {}",
                id
            );
        }
    }

    #[test]
    fn player_deck_weighted_selection_respects_probabilities() {
        // Create a deck with heavily weighted first card
        let cards = vec![
            DeckCard::creature("common", 90.0),    // 90% weight
            DeckCard::creature("rare", 10.0),      // 10% weight
        ];
        let deck = PlayerDeck::new(cards);

        let mut common_count = 0;
        let mut rare_count = 0;
        let iterations = 1000;

        for _ in 0..iterations {
            let rolled = deck.roll_card().unwrap();
            if rolled.id == "common" {
                common_count += 1;
            } else {
                rare_count += 1;
            }
        }

        // With 90/10 split, common should be significantly more frequent
        // Allow for statistical variance - common should be at least 80% of rolls
        let common_ratio = common_count as f64 / iterations as f64;
        assert!(
            common_ratio > 0.80,
            "Common card should appear ~90% of time, got {:.1}%",
            common_ratio * 100.0
        );
        assert!(
            rare_count > 0,
            "Rare card should appear at least once in {} iterations",
            iterations
        );
    }

    #[test]
    fn player_deck_handles_negative_weights_gracefully() {
        // Negative weights shouldn't crash, but behavior is undefined
        // This test ensures no panic
        let cards = vec![
            DeckCard::creature("negative", -5.0),
            DeckCard::creature("positive", 10.0),
        ];
        let deck = PlayerDeck::new(cards);
        // Total weight would be 5.0
        assert_eq!(deck.total_weight, 5.0);
        // Rolling should work without panicking
        let _ = deck.roll_card();
    }

    #[test]
    fn card_type_equality_works() {
        assert_eq!(CardType::Creature, CardType::Creature);
        assert_eq!(CardType::Weapon, CardType::Weapon);
        assert_eq!(CardType::Artifact, CardType::Artifact);
        assert_ne!(CardType::Creature, CardType::Weapon);
        assert_ne!(CardType::Weapon, CardType::Artifact);
    }
}
