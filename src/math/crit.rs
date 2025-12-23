use rand::Rng;

/// Maximum damage cap to prevent overflow (1e15)
pub const MAX_DAMAGE_CAP: f64 = 1e15;

/// Crit tier levels
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CritTier {
    #[default]
    None,
    /// Tier 1: Normal Crit - 2x damage (overflow adds bonus)
    Normal,
    /// Tier 2: Mega Crit - damage squared
    Mega,
    /// Tier 3: Super Crit - damage^4 (capped at 1e15)
    Super,
}

impl CritTier {
    /// Returns a numeric value for ordering (higher = better)
    pub fn tier_value(&self) -> u8 {
        match self {
            CritTier::None => 0,
            CritTier::Normal => 1,
            CritTier::Mega => 2,
            CritTier::Super => 3,
        }
    }
}

/// Result of a damage calculation with crits
#[derive(Clone, Debug)]
pub struct CritResult {
    /// The crit tier that was applied
    pub tier: CritTier,
    /// The final damage after crit modifiers
    pub final_damage: f64,
    /// The original base damage before crits
    pub base_damage: f64,
}

impl CritResult {
    pub fn new(tier: CritTier, final_damage: f64, base_damage: f64) -> Self {
        Self {
            tier,
            final_damage,
            base_damage,
        }
    }

    /// Check if this was a crit of any tier
    pub fn is_crit(&self) -> bool {
        self.tier != CritTier::None
    }
}

/// Calculate damage with crit chances for all three tiers.
///
/// Each tier is rolled independently. The highest successful tier wins.
///
/// Crit mechanics:
/// - Tier 1 (Normal): 2x damage. If crit_t1 > 100%, overflow adds bonus damage.
///   e.g., 150% crit_t1 = guaranteed crit + 50% chance of extra 2x
/// - Tier 2 (Mega): damage squared
/// - Tier 3 (Super): damage^4, capped at 1e15
///
/// # Arguments
/// * `base_damage` - The base damage before any crit modifiers
/// * `crit_t1` - Tier 1 crit chance as percentage (e.g., 5.0 = 5%)
/// * `crit_t2` - Tier 2 crit chance as percentage (e.g., 1.0 = 1%)
/// * `crit_t3` - Tier 3 crit chance as percentage (e.g., 0.1 = 0.1%)
pub fn calculate_damage_with_crits(
    base_damage: f64,
    crit_t1: f64,
    crit_t2: f64,
    crit_t3: f64,
) -> CritResult {
    let mut rng = rand::thread_rng();

    // Roll each tier independently
    let t1_roll: f64 = rng.gen_range(0.0..100.0);
    let t2_roll: f64 = rng.gen_range(0.0..100.0);
    let t3_roll: f64 = rng.gen_range(0.0..100.0);

    let t1_success = t1_roll < crit_t1;
    let t2_success = t2_roll < crit_t2;
    let t3_success = t3_roll < crit_t3;

    // Determine highest tier hit
    let tier = if t3_success {
        CritTier::Super
    } else if t2_success {
        CritTier::Mega
    } else if t1_success {
        CritTier::Normal
    } else {
        CritTier::None
    };

    // Calculate final damage based on tier
    let final_damage = match tier {
        CritTier::None => base_damage,
        CritTier::Normal => {
            // 2x base damage
            let mut damage = base_damage * 2.0;

            // Handle overflow: if crit_t1 > 100%, extra chance for additional 2x
            if crit_t1 > 100.0 {
                let overflow = crit_t1 - 100.0;
                let overflow_roll: f64 = rng.gen_range(0.0..100.0);
                if overflow_roll < overflow {
                    damage *= 2.0; // Extra 2x from overflow
                }
            }
            damage
        }
        CritTier::Mega => {
            // Damage squared
            base_damage * base_damage
        }
        CritTier::Super => {
            // Damage^4, capped at 1e15
            let damage = base_damage.powi(4);
            damage.min(MAX_DAMAGE_CAP)
        }
    };

    CritResult::new(tier, final_damage, base_damage)
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // CritTier Tests
    // =========================================================================

    #[test]
    fn crit_tier_default_is_none() {
        assert_eq!(CritTier::default(), CritTier::None);
    }

    #[test]
    fn crit_tier_value_ordering() {
        assert!(CritTier::Super.tier_value() > CritTier::Mega.tier_value());
        assert!(CritTier::Mega.tier_value() > CritTier::Normal.tier_value());
        assert!(CritTier::Normal.tier_value() > CritTier::None.tier_value());
    }

    #[test]
    fn crit_tier_values_are_correct() {
        assert_eq!(CritTier::None.tier_value(), 0);
        assert_eq!(CritTier::Normal.tier_value(), 1);
        assert_eq!(CritTier::Mega.tier_value(), 2);
        assert_eq!(CritTier::Super.tier_value(), 3);
    }

    // =========================================================================
    // CritResult Tests
    // =========================================================================

    #[test]
    fn crit_result_is_crit_returns_false_for_none() {
        let result = CritResult::new(CritTier::None, 100.0, 100.0);
        assert!(!result.is_crit());
    }

    #[test]
    fn crit_result_is_crit_returns_true_for_all_crit_tiers() {
        let normal = CritResult::new(CritTier::Normal, 200.0, 100.0);
        let mega = CritResult::new(CritTier::Mega, 10000.0, 100.0);
        let super_crit = CritResult::new(CritTier::Super, 100000000.0, 100.0);

        assert!(normal.is_crit());
        assert!(mega.is_crit());
        assert!(super_crit.is_crit());
    }

    #[test]
    fn crit_result_preserves_values() {
        let result = CritResult::new(CritTier::Mega, 2500.0, 50.0);
        assert_eq!(result.tier, CritTier::Mega);
        assert_eq!(result.final_damage, 2500.0);
        assert_eq!(result.base_damage, 50.0);
    }

    // =========================================================================
    // calculate_damage_with_crits Tests
    // =========================================================================

    #[test]
    fn no_crit_with_zero_chances() {
        // With 0% crit chances, should never crit
        for _ in 0..100 {
            let result = calculate_damage_with_crits(100.0, 0.0, 0.0, 0.0);
            assert_eq!(result.tier, CritTier::None);
            assert_eq!(result.final_damage, 100.0);
            assert_eq!(result.base_damage, 100.0);
        }
    }

    #[test]
    fn guaranteed_t1_crit_with_100_percent() {
        // 100% T1 crit chance should always crit (but not T2 or T3)
        for _ in 0..100 {
            let result = calculate_damage_with_crits(100.0, 100.0, 0.0, 0.0);
            // Should be at least Normal tier
            assert!(result.is_crit());
            // T1 gives 2x damage
            assert!(result.final_damage >= 200.0);
        }
    }

    #[test]
    fn guaranteed_t2_crit_with_100_percent() {
        // 100% T2 crit chance should hit Mega
        for _ in 0..100 {
            let result = calculate_damage_with_crits(10.0, 0.0, 100.0, 0.0);
            assert_eq!(result.tier, CritTier::Mega);
            // Mega = damage squared = 10^2 = 100
            assert_eq!(result.final_damage, 100.0);
        }
    }

    #[test]
    fn guaranteed_t3_crit_with_100_percent() {
        // 100% T3 crit chance should hit Super
        for _ in 0..100 {
            let result = calculate_damage_with_crits(10.0, 0.0, 0.0, 100.0);
            assert_eq!(result.tier, CritTier::Super);
            // Super = damage^4 = 10^4 = 10000
            assert_eq!(result.final_damage, 10000.0);
        }
    }

    #[test]
    fn t3_wins_over_lower_tiers() {
        // When all crits hit, T3 (Super) should be the result
        for _ in 0..100 {
            let result = calculate_damage_with_crits(10.0, 100.0, 100.0, 100.0);
            assert_eq!(result.tier, CritTier::Super);
        }
    }

    #[test]
    fn t2_wins_over_t1_when_t3_misses() {
        // T2 and T1 both crit, T3 misses - should be Mega
        for _ in 0..100 {
            let result = calculate_damage_with_crits(10.0, 100.0, 100.0, 0.0);
            assert_eq!(result.tier, CritTier::Mega);
        }
    }

    #[test]
    fn super_crit_capped_at_1e15() {
        // Large base damage should cap at 1e15
        let result = calculate_damage_with_crits(100000.0, 0.0, 0.0, 100.0);
        // 100000^4 = 1e20, should be capped to 1e15
        assert_eq!(result.final_damage, MAX_DAMAGE_CAP);
    }

    #[test]
    fn mega_crit_squares_damage() {
        let result = calculate_damage_with_crits(50.0, 0.0, 100.0, 0.0);
        // 50^2 = 2500
        assert_eq!(result.final_damage, 2500.0);
    }

    #[test]
    fn normal_crit_doubles_damage() {
        // Need to run multiple times since T1 has randomness in overflow
        let result = calculate_damage_with_crits(50.0, 100.0, 0.0, 0.0);
        // At minimum, 2x damage
        assert!(result.final_damage >= 100.0);
        // Since there's no overflow, should be exactly 2x
        assert_eq!(result.final_damage, 100.0);
    }

    #[test]
    fn base_damage_preserved() {
        let result = calculate_damage_with_crits(123.456, 0.0, 0.0, 0.0);
        assert_eq!(result.base_damage, 123.456);
    }

    #[test]
    fn overflow_crit_can_add_bonus() {
        // With 200% T1 crit, guaranteed crit + 100% chance of extra 2x
        // So should always be 4x damage
        for _ in 0..100 {
            let result = calculate_damage_with_crits(25.0, 200.0, 0.0, 0.0);
            assert_eq!(result.tier, CritTier::Normal);
            // 25 * 2 * 2 = 100
            assert_eq!(result.final_damage, 100.0);
        }
    }

    #[test]
    fn statistical_crit_distribution() {
        // With 50% crit chance, roughly half should crit
        let mut crits = 0;
        let iterations = 10000;

        for _ in 0..iterations {
            let result = calculate_damage_with_crits(100.0, 50.0, 0.0, 0.0);
            if result.is_crit() {
                crits += 1;
            }
        }

        // Should be roughly 50%, allow for statistical variance (40-60%)
        let crit_rate = crits as f64 / iterations as f64;
        assert!(crit_rate > 0.40 && crit_rate < 0.60,
            "Expected ~50% crit rate, got {:.2}%", crit_rate * 100.0);
    }
}
