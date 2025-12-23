use bevy::prelude::*;
use std::collections::HashSet;

/// Pool size for projectiles
pub const PROJECTILE_POOL_SIZE: usize = 5000;

/// Pool size for damage numbers
pub const DAMAGE_NUMBER_POOL_SIZE: usize = 500;

/// Pool of pre-allocated projectile entities for reuse
#[derive(Resource)]
pub struct ProjectilePool {
    /// Entities available for use
    pub available: Vec<Entity>,
    /// Entities currently in use
    pub active: HashSet<Entity>,
}

impl Default for ProjectilePool {
    fn default() -> Self {
        Self {
            available: Vec::with_capacity(PROJECTILE_POOL_SIZE),
            active: HashSet::with_capacity(PROJECTILE_POOL_SIZE),
        }
    }
}

impl ProjectilePool {
    /// Get an entity from the pool, or None if pool is empty
    pub fn get(&mut self) -> Option<Entity> {
        if let Some(entity) = self.available.pop() {
            self.active.insert(entity);
            Some(entity)
        } else {
            None
        }
    }

    /// Return an entity to the pool
    pub fn release(&mut self, entity: Entity) {
        if self.active.remove(&entity) {
            self.available.push(entity);
        }
    }

    /// Check if pool has available entities
    pub fn has_available(&self) -> bool {
        !self.available.is_empty()
    }

    /// Get count of available entities
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// Get count of active entities
    pub fn active_count(&self) -> usize {
        self.active.len()
    }
}

/// Pool of pre-allocated damage number entities for reuse
#[derive(Resource)]
pub struct DamageNumberPool {
    /// Entities available for use
    pub available: Vec<Entity>,
    /// Entities currently in use
    pub active: HashSet<Entity>,
}

impl Default for DamageNumberPool {
    fn default() -> Self {
        Self {
            available: Vec::with_capacity(DAMAGE_NUMBER_POOL_SIZE),
            active: HashSet::with_capacity(DAMAGE_NUMBER_POOL_SIZE),
        }
    }
}

impl DamageNumberPool {
    /// Get an entity from the pool, or None if pool is empty
    pub fn get(&mut self) -> Option<Entity> {
        if let Some(entity) = self.available.pop() {
            self.active.insert(entity);
            Some(entity)
        } else {
            None
        }
    }

    /// Return an entity to the pool
    pub fn release(&mut self, entity: Entity) {
        if self.active.remove(&entity) {
            self.available.push(entity);
        }
    }

    /// Check if pool has available entities
    pub fn has_available(&self) -> bool {
        !self.available.is_empty()
    }

    /// Get count of available entities
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// Get count of active entities
    pub fn active_count(&self) -> usize {
        self.active.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projectile_pool_get_and_release() {
        let mut pool = ProjectilePool::default();
        let entity = Entity::from_raw(1);
        pool.available.push(entity);

        assert_eq!(pool.available_count(), 1);
        assert_eq!(pool.active_count(), 0);

        let gotten = pool.get();
        assert_eq!(gotten, Some(entity));
        assert_eq!(pool.available_count(), 0);
        assert_eq!(pool.active_count(), 1);

        pool.release(entity);
        assert_eq!(pool.available_count(), 1);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn projectile_pool_returns_none_when_empty() {
        let mut pool = ProjectilePool::default();
        assert_eq!(pool.get(), None);
    }

    #[test]
    fn damage_number_pool_get_and_release() {
        let mut pool = DamageNumberPool::default();
        let entity = Entity::from_raw(1);
        pool.available.push(entity);

        let gotten = pool.get();
        assert_eq!(gotten, Some(entity));

        pool.release(entity);
        assert!(pool.has_available());
    }
}
