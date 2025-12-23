use bevy::prelude::*;
use std::collections::HashMap;

/// Cell size for spatial grid (in pixels)
pub const SPATIAL_CELL_SIZE: f32 = 256.0;

/// Spatial grid for efficient entity lookups
/// Divides the world into cells and tracks which enemies are in each cell
#[derive(Resource, Default)]
pub struct SpatialGrid {
    /// Map from cell coordinates to list of enemy entities in that cell
    cells: HashMap<(i32, i32), Vec<Entity>>,
}

impl SpatialGrid {
    /// Clear all cells
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Get cell coordinates for a world position
    pub fn get_cell(pos: Vec2) -> (i32, i32) {
        (
            (pos.x / SPATIAL_CELL_SIZE).floor() as i32,
            (pos.y / SPATIAL_CELL_SIZE).floor() as i32,
        )
    }

    /// Insert an entity at the given position
    pub fn insert(&mut self, entity: Entity, pos: Vec2) {
        let cell = Self::get_cell(pos);
        self.cells.entry(cell).or_default().push(entity);
    }

    /// Get all entities in a cell
    pub fn get_entities_in_cell(&self, cell: (i32, i32)) -> &[Entity] {
        self.cells.get(&cell).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get all entities in a cell and its 8 neighbors (3x3 area)
    pub fn get_nearby_entities(&self, pos: Vec2) -> Vec<Entity> {
        let (cx, cy) = Self::get_cell(pos);
        let mut result = Vec::new();

        // Check 3x3 grid of cells centered on the position's cell
        for dx in -1..=1 {
            for dy in -1..=1 {
                let cell = (cx + dx, cy + dy);
                if let Some(entities) = self.cells.get(&cell) {
                    result.extend(entities.iter().copied());
                }
            }
        }

        result
    }

    /// Get entities within a radius (checks all cells that could contain entities in range)
    pub fn get_entities_in_radius(&self, pos: Vec2, radius: f32) -> Vec<Entity> {
        let cells_to_check = (radius / SPATIAL_CELL_SIZE).ceil() as i32 + 1;
        let (cx, cy) = Self::get_cell(pos);
        let mut result = Vec::new();

        for dx in -cells_to_check..=cells_to_check {
            for dy in -cells_to_check..=cells_to_check {
                let cell = (cx + dx, cy + dy);
                if let Some(entities) = self.cells.get(&cell) {
                    result.extend(entities.iter().copied());
                }
            }
        }

        result
    }

    /// Get number of cells with entities
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Get total entities tracked
    pub fn entity_count(&self) -> usize {
        self.cells.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_cell_calculates_correct_coordinates() {
        // Origin is cell (0, 0)
        assert_eq!(SpatialGrid::get_cell(Vec2::new(0.0, 0.0)), (0, 0));
        assert_eq!(SpatialGrid::get_cell(Vec2::new(100.0, 100.0)), (0, 0));

        // Positive coordinates
        assert_eq!(SpatialGrid::get_cell(Vec2::new(256.0, 0.0)), (1, 0));
        assert_eq!(SpatialGrid::get_cell(Vec2::new(512.0, 512.0)), (2, 2));

        // Negative coordinates
        assert_eq!(SpatialGrid::get_cell(Vec2::new(-1.0, 0.0)), (-1, 0));
        assert_eq!(SpatialGrid::get_cell(Vec2::new(-256.0, -256.0)), (-1, -1));
    }

    #[test]
    fn insert_and_retrieve_entities() {
        let mut grid = SpatialGrid::default();
        let entity = Entity::from_raw(1);

        grid.insert(entity, Vec2::new(100.0, 100.0));

        let entities = grid.get_entities_in_cell((0, 0));
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0], entity);
    }

    #[test]
    fn get_nearby_returns_entities_from_adjacent_cells() {
        let mut grid = SpatialGrid::default();

        // Insert entities in different cells
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);

        grid.insert(e1, Vec2::new(0.0, 0.0));      // Cell (0, 0)
        grid.insert(e2, Vec2::new(256.0, 0.0));    // Cell (1, 0)
        grid.insert(e3, Vec2::new(1000.0, 1000.0)); // Cell (3, 3) - far away

        // Get nearby from origin - should include e1 and e2, but not e3
        let nearby = grid.get_nearby_entities(Vec2::new(128.0, 128.0));
        assert!(nearby.contains(&e1));
        assert!(nearby.contains(&e2));
        assert!(!nearby.contains(&e3));
    }

    #[test]
    fn clear_removes_all_entities() {
        let mut grid = SpatialGrid::default();
        grid.insert(Entity::from_raw(1), Vec2::new(0.0, 0.0));
        grid.insert(Entity::from_raw(2), Vec2::new(100.0, 100.0));

        assert!(grid.entity_count() > 0);

        grid.clear();

        assert_eq!(grid.entity_count(), 0);
        assert_eq!(grid.cell_count(), 0);
    }
}
