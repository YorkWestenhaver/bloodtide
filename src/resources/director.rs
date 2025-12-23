use bevy::prelude::*;

/// Director AI resource - controls spawn rates and adapts to player performance
/// Designed for MASSIVE horde spawning (Vampire Survivors-style)
#[derive(Resource)]
pub struct Director {
    /// Estimated player DPS based on recent damage dealt
    pub player_dps: f64,
    /// Number of active creatures
    pub creature_count: u32,
    /// Average HP percentage of all creatures (0.0 - 1.0)
    pub total_creature_hp_percent: f64,
    /// Current stress level (0.0 = easy, 1.0 = struggling)
    pub stress_level: f32,
    /// Number of enemies currently alive
    pub enemies_alive: u32,
    /// Modifier applied to spawn rate (higher = more spawns)
    pub spawn_rate_modifier: f32,
    /// Rolling damage tracker for DPS calculation
    pub damage_dealt_window: Vec<(f64, f32)>, // (damage, timestamp)
    /// Current FPS for performance monitoring
    pub current_fps: f32,
    /// How long FPS has been low
    pub low_fps_duration: f32,
    /// Performance throttle multiplier (1.0 = normal, 0.5 = halved spawns)
    pub performance_throttle: f32,
}

impl Default for Director {
    fn default() -> Self {
        Self {
            player_dps: 0.0,
            creature_count: 0,
            total_creature_hp_percent: 1.0,
            stress_level: 0.5,
            enemies_alive: 0,
            spawn_rate_modifier: 1.0,
            damage_dealt_window: Vec::new(),
            current_fps: 60.0,
            low_fps_duration: 0.0,
            performance_throttle: 1.0,
        }
    }
}

impl Director {
    /// Get target enemy count for current wave - LINEAR scaling for early game
    pub fn get_target_enemy_count(wave: u32) -> u32 {
        match wave {
            // Very gentle start - linear scaling from wave 1-10
            1 => 15,           // Just a handful to start
            2 => 25,
            3 => 40,
            4 => 60,
            5 => 85,
            6 => 115,
            7 => 150,
            8 => 190,
            9 => 235,
            10 => 285,
            // Start ramping up more after wave 10
            11..=15 => 300 + (wave - 10) * 100,  // 400-800
            16..=20 => 800 + (wave - 15) * 200,  // 1000-1800
            21..=30 => 1800 + (wave - 20) * 300, // 2100-4800
            _ => 5000 + (wave - 30) * 100,       // 5000+ scaling
        }
    }

    /// Get enemies to spawn per spawn event based on wave - MUCH gentler early game
    pub fn get_enemies_per_spawn(wave: u32) -> (u32, u32) {
        match wave {
            // Very small spawns early - let player build up
            1 => (2, 4),       // Just a trickle
            2 => (3, 5),
            3 => (4, 7),
            4 => (5, 9),
            5 => (6, 11),
            6 => (8, 14),
            7 => (10, 17),
            8 => (12, 20),
            9 => (15, 25),
            10 => (18, 30),
            // Now start ramping up
            11..=15 => (25, 45),
            16..=20 => (50, 90),
            21..=30 => (100, 180),
            _ => (200, 350),
        }
    }

    /// Get elite spawn chance for current wave
    pub fn get_elite_chance(wave: u32) -> f32 {
        match wave {
            1..=5 => 0.02,
            6..=10 => 0.05,
            11..=15 => 0.10,
            16..=20 => 0.15,
            _ => 0.20,
        }
    }

    /// Get spawn interval based on enemy count vs target
    pub fn get_spawn_interval(&self, wave: u32) -> f32 {
        let target = Self::get_target_enemy_count(wave) as f32;
        let ratio = self.enemies_alive as f32 / target.max(1.0);

        // Base interval depends on wave - early waves spawn slower
        let wave_base = match wave {
            1..=3 => 1.5,    // Very slow early - give player time
            4..=6 => 1.0,    // Still gentle
            7..=10 => 0.7,   // Starting to pick up
            11..=15 => 0.4,  // Getting serious
            _ => 0.2,        // Late game - fast spawns
        };

        // Adjust based on how close we are to target
        let ratio_modifier = if ratio < 0.5 {
            0.7  // Below target - spawn a bit faster
        } else if ratio < 1.0 {
            1.0  // Approaching target - normal
        } else if ratio < 1.5 {
            1.5  // At target - slow down
        } else {
            2.5  // Over target - spawn much slower
        };

        // Apply stress modifier - but less aggressive in early waves
        let stress_modifier = if wave <= 5 {
            // Early waves: minimal stress adjustment
            match self.stress_level {
                s if s > 0.8 => 1.3,  // Only slow down if really struggling
                _ => 1.0,
            }
        } else {
            // Later waves: normal stress adjustment
            match self.stress_level {
                s if s < 0.3 => 0.7,  // Stomping - spawn faster
                s if s > 0.7 => 1.5,  // Struggling - spawn slower
                _ => 1.0,             // Comfortable
            }
        };

        // Apply performance throttle
        let interval = wave_base * ratio_modifier * stress_modifier / self.spawn_rate_modifier * (1.0 / self.performance_throttle);

        // Clamp to reasonable range
        interval.clamp(0.15, 3.0)
    }

    /// Get HP scaling modifier for current wave
    pub fn get_hp_scale(wave: u32) -> f64 {
        // Slower scaling since there are WAY more enemies
        1.0 + (wave as f64 - 1.0) * 0.08
    }

    /// Calculate stress level based on current metrics
    pub fn calculate_stress(&mut self) {
        // Stress factors:
        // - Low creature HP = more stress
        // - Low creature count = more stress
        // - High enemy count = more stress

        let hp_stress = 1.0 - self.total_creature_hp_percent as f32;
        let creature_stress = if self.creature_count == 0 {
            1.0
        } else {
            (1.0 / self.creature_count as f32).min(1.0)
        };
        let enemy_stress = (self.enemies_alive as f32 / 1000.0).min(1.0);

        // Weighted average
        self.stress_level = (hp_stress * 0.4 + creature_stress * 0.3 + enemy_stress * 0.3).clamp(0.0, 1.0);
    }

    /// Update DPS calculation from damage window
    pub fn update_dps(&mut self, current_time: f32) {
        // Remove old entries (older than 3 seconds)
        self.damage_dealt_window.retain(|(_, time)| current_time - time < 3.0);

        // Calculate DPS
        let total_damage: f64 = self.damage_dealt_window.iter().map(|(d, _)| d).sum();
        self.player_dps = total_damage / 3.0;
    }

    /// Record damage dealt
    pub fn record_damage(&mut self, damage: f64, timestamp: f32) {
        self.damage_dealt_window.push((damage, timestamp));
    }

    /// Update performance throttle based on FPS
    pub fn update_performance(&mut self, fps: f32, delta: f32) {
        self.current_fps = fps;

        if fps < 30.0 {
            self.low_fps_duration += delta;
            if self.low_fps_duration > 3.0 {
                // FPS low for 3+ seconds - reduce spawns
                if fps < 20.0 {
                    self.performance_throttle = 0.5;
                    // Only print warning once
                    if self.low_fps_duration < 3.1 {
                        println!("WARNING: Low FPS ({:.0}) - reducing spawn rate by 50%", fps);
                    }
                } else {
                    self.performance_throttle = 0.75;
                }
            }
        } else if fps > 45.0 {
            // FPS recovered
            self.low_fps_duration = 0.0;
            self.performance_throttle = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn director_default_values() {
        let director = Director::default();
        assert_eq!(director.enemies_alive, 0);
        assert_eq!(director.spawn_rate_modifier, 1.0);
        assert_eq!(director.performance_throttle, 1.0);
    }

    #[test]
    fn target_enemy_count_scales_with_wave() {
        assert!(Director::get_target_enemy_count(1) < Director::get_target_enemy_count(10));
        assert!(Director::get_target_enemy_count(10) < Director::get_target_enemy_count(20));
        assert!(Director::get_target_enemy_count(20) < Director::get_target_enemy_count(30));
    }

    #[test]
    fn enemies_per_spawn_scales_with_wave() {
        let (min1, _max1) = Director::get_enemies_per_spawn(1);
        let (min10, _max10) = Director::get_enemies_per_spawn(10);
        let (min20, _max20) = Director::get_enemies_per_spawn(20);

        // Later waves have higher minimums
        assert!(min1 < min10);
        assert!(min10 < min20);
    }

    #[test]
    fn elite_chance_increases_with_wave() {
        assert!(Director::get_elite_chance(1) < Director::get_elite_chance(10));
        assert!(Director::get_elite_chance(10) < Director::get_elite_chance(20));
    }

    #[test]
    fn hp_scale_increases_with_wave() {
        assert!(Director::get_hp_scale(1) < Director::get_hp_scale(10));
        assert!(Director::get_hp_scale(10) < Director::get_hp_scale(20));
    }

    #[test]
    fn spawn_interval_faster_when_below_target() {
        let mut director = Director::default();
        director.enemies_alive = 100;
        let interval_low = director.get_spawn_interval(10);

        director.enemies_alive = 2000;
        let interval_high = director.get_spawn_interval(10);

        assert!(interval_low < interval_high);
    }
}
