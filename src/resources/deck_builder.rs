use bevy::prelude::*;

use crate::resources::deck::{CardType, DeckCard, PlayerDeck};

/// Currently selected tab in the deck builder UI
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CardTab {
    #[default]
    Creatures,
    Weapons,
    Artifacts,
}

/// A card entry in the deck builder (with copy count instead of weight)
#[derive(Clone, Debug)]
pub struct DeckBuilderCard {
    pub card_type: CardType,
    pub id: String,
    pub copies: u32,
}

impl DeckBuilderCard {
    pub fn new(card_type: CardType, id: &str, copies: u32) -> Self {
        Self {
            card_type,
            id: id.to_string(),
            copies,
        }
    }

    pub fn creature(id: &str, copies: u32) -> Self {
        Self::new(CardType::Creature, id, copies)
    }

    pub fn weapon(id: &str, copies: u32) -> Self {
        Self::new(CardType::Weapon, id, copies)
    }

    pub fn artifact(id: &str, copies: u32) -> Self {
        Self::new(CardType::Artifact, id, copies)
    }
}

/// Working deck state during deck builder editing
#[derive(Resource)]
pub struct DeckBuilderState {
    pub cards: Vec<DeckBuilderCard>,
    pub selected_tab: CardTab,
    /// Selected starting weapon (weapon id)
    pub starting_weapon: Option<String>,
}

impl Default for DeckBuilderState {
    fn default() -> Self {
        // Default starter deck
        Self {
            cards: vec![
                DeckBuilderCard::creature("fire_imp", 5),
                DeckBuilderCard::creature("ember_hound", 3),
                DeckBuilderCard::weapon("ember_staff", 3),
                DeckBuilderCard::artifact("molten_core", 2),
            ],
            selected_tab: CardTab::Creatures,
            starting_weapon: Some("ember_staff".to_string()),
        }
    }
}

impl DeckBuilderState {
    /// Add a card to the deck (or increment copies if exists)
    pub fn add_card(&mut self, card_type: CardType, id: &str) {
        if let Some(card) = self.cards.iter_mut().find(|c| c.id == id) {
            card.copies = (card.copies + 1).min(10);
        } else {
            self.cards.push(DeckBuilderCard::new(card_type, id, 1));
        }
    }

    /// Remove a copy of a card (removes card entirely if copies reaches 0)
    pub fn remove_card(&mut self, id: &str) {
        if let Some(pos) = self.cards.iter().position(|c| c.id == id) {
            if self.cards[pos].copies > 1 {
                self.cards[pos].copies -= 1;
            } else {
                self.cards.remove(pos);
            }
        }
    }

    /// Increment copies of a card
    pub fn increment_copies(&mut self, id: &str) {
        if let Some(card) = self.cards.iter_mut().find(|c| c.id == id) {
            card.copies = (card.copies + 1).min(10);
        }
    }

    /// Decrement copies of a card (removes if reaches 0)
    pub fn decrement_copies(&mut self, id: &str) {
        self.remove_card(id);
    }

    /// Get total copies in deck
    pub fn total_copies(&self) -> u32 {
        self.cards.iter().map(|c| c.copies).sum()
    }

    /// Get probability of a card being rolled (as percentage 0-100)
    pub fn get_probability(&self, id: &str) -> f32 {
        let total = self.total_copies();
        if total == 0 {
            return 0.0;
        }
        if let Some(card) = self.cards.iter().find(|c| c.id == id) {
            (card.copies as f32 / total as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Check if a card is in the deck
    pub fn has_card(&self, id: &str) -> bool {
        self.cards.iter().any(|c| c.id == id)
    }

    /// Clear all cards from the deck
    pub fn clear(&mut self) {
        self.cards.clear();
    }

    /// Check if deck is empty
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Convert to PlayerDeck for gameplay (copies become weight)
    pub fn to_player_deck(&self) -> PlayerDeck {
        let cards: Vec<DeckCard> = self
            .cards
            .iter()
            .map(|c| DeckCard {
                card_type: c.card_type.clone(),
                id: c.id.clone(),
                weight: c.copies as f64 * 5.0, // Each copy = 5 weight
            })
            .collect();
        PlayerDeck::new(cards)
    }

    /// Get cards filtered by type
    pub fn cards_by_type(&self, card_type: CardType) -> Vec<&DeckBuilderCard> {
        self.cards
            .iter()
            .filter(|c| c.card_type == card_type)
            .collect()
    }

    /// Get type breakdown percentages
    pub fn type_breakdown(&self) -> (f32, f32, f32) {
        let total = self.total_copies() as f32;
        if total == 0.0 {
            return (0.0, 0.0, 0.0);
        }
        let creatures: u32 = self
            .cards
            .iter()
            .filter(|c| c.card_type == CardType::Creature)
            .map(|c| c.copies)
            .sum();
        let weapons: u32 = self
            .cards
            .iter()
            .filter(|c| c.card_type == CardType::Weapon)
            .map(|c| c.copies)
            .sum();
        let artifacts: u32 = self
            .cards
            .iter()
            .filter(|c| c.card_type == CardType::Artifact)
            .map(|c| c.copies)
            .sum();
        (
            creatures as f32 / total * 100.0,
            weapons as f32 / total * 100.0,
            artifacts as f32 / total * 100.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_deck_builder_has_starter_cards() {
        let state = DeckBuilderState::default();
        assert!(!state.is_empty());
        assert!(state.has_card("fire_imp"));
        assert!(state.has_card("ember_hound"));
        assert!(state.has_card("ember_staff"));
        assert!(state.has_card("molten_core"));
    }

    #[test]
    fn add_new_card() {
        let mut state = DeckBuilderState { cards: vec![], selected_tab: CardTab::Creatures, starting_weapon: None };
        state.add_card(CardType::Creature, "fire_imp");
        assert!(state.has_card("fire_imp"));
        assert_eq!(state.cards[0].copies, 1);
    }

    #[test]
    fn add_existing_card_increments_copies() {
        let mut state = DeckBuilderState { cards: vec![], selected_tab: CardTab::Creatures, starting_weapon: None };
        state.add_card(CardType::Creature, "fire_imp");
        state.add_card(CardType::Creature, "fire_imp");
        assert_eq!(state.cards.len(), 1);
        assert_eq!(state.cards[0].copies, 2);
    }

    #[test]
    fn copies_capped_at_10() {
        let mut state = DeckBuilderState { cards: vec![], selected_tab: CardTab::Creatures, starting_weapon: None };
        for _ in 0..15 {
            state.add_card(CardType::Creature, "fire_imp");
        }
        assert_eq!(state.cards[0].copies, 10);
    }

    #[test]
    fn remove_card_decrements_copies() {
        let mut state = DeckBuilderState { cards: vec![], selected_tab: CardTab::Creatures, starting_weapon: None };
        state.add_card(CardType::Creature, "fire_imp");
        state.add_card(CardType::Creature, "fire_imp");
        state.remove_card("fire_imp");
        assert_eq!(state.cards[0].copies, 1);
    }

    #[test]
    fn remove_card_removes_at_zero() {
        let mut state = DeckBuilderState { cards: vec![], selected_tab: CardTab::Creatures, starting_weapon: None };
        state.add_card(CardType::Creature, "fire_imp");
        state.remove_card("fire_imp");
        assert!(!state.has_card("fire_imp"));
        assert!(state.is_empty());
    }

    #[test]
    fn probability_calculation() {
        let mut state = DeckBuilderState { cards: vec![], selected_tab: CardTab::Creatures, starting_weapon: None };
        state.add_card(CardType::Creature, "fire_imp");
        state.add_card(CardType::Creature, "fire_imp");
        state.add_card(CardType::Creature, "ember_hound");
        // 2 fire_imp, 1 ember_hound = 3 total
        // fire_imp probability = 2/3 * 100 = 66.67%
        let prob = state.get_probability("fire_imp");
        assert!((prob - 66.67).abs() < 0.1);
    }

    #[test]
    fn to_player_deck_conversion() {
        let mut state = DeckBuilderState { cards: vec![], selected_tab: CardTab::Creatures, starting_weapon: None };
        state.add_card(CardType::Creature, "fire_imp");
        state.add_card(CardType::Creature, "fire_imp");
        let deck = state.to_player_deck();
        assert_eq!(deck.cards.len(), 1);
        assert_eq!(deck.cards[0].weight, 10.0); // 2 copies * 5 weight
    }

    #[test]
    fn type_breakdown() {
        let state = DeckBuilderState {
            cards: vec![
                DeckBuilderCard::creature("a", 2),
                DeckBuilderCard::weapon("b", 1),
                DeckBuilderCard::artifact("c", 1),
            ],
            selected_tab: CardTab::Creatures,
            starting_weapon: None,
        };
        let (creatures, weapons, artifacts) = state.type_breakdown();
        assert!((creatures - 50.0).abs() < 0.1);
        assert!((weapons - 25.0).abs() < 0.1);
        assert!((artifacts - 25.0).abs() < 0.1);
    }

    #[test]
    fn clear_removes_all() {
        let mut state = DeckBuilderState::default();
        state.clear();
        assert!(state.is_empty());
    }
}
