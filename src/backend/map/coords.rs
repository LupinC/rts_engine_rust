use bevy::prelude::*;

/// Size of a single isometric tile in pixels.
#[derive(Debug, Clone, Copy)]
pub struct IsoTileSize {
    pub w: f32,
    pub h: f32,
}

impl IsoTileSize {
    pub const fn new(w: f32, h: f32) -> Self {
        Self { w, h }
    }
}

/// Helper that converts between grid indices (i, j) and world/screen coordinates for
/// a staggered rectangular isometric layout (odd-row offset).
#[derive(Debug, Clone, Copy)]
pub struct IsoStaggered {
    pub tile: IsoTileSize,
    pub origin: Vec2,
}

impl IsoStaggered {
    /// Compute the top-left corner of the tile's bounding box in world coordinates.
    pub fn world_from_ij(&self, i: i32, j: i32) -> Vec2 {
        let tw = self.tile.w;
        let dy = self.tile.h * 0.5;
        let shift = if i & 1 == 1 { tw * 0.5 } else { 0.0 };

        Vec2::new(
            self.origin.x + j as f32 * tw + shift,
            self.origin.y + i as f32 * dy,
        )
    }

    /// Center point (intersection of diagonals) of a tile.
    pub fn world_center(&self, i: i32, j: i32) -> Vec2 {
        let base = self.world_from_ij(i, j);
        Vec2::new(base.x + self.tile.w * 0.5, base.y + self.tile.h * 0.5)
    }

    /// Vertices of the tile rhombus (top, right, bottom, left order).
    pub fn tile_corners(&self, i: i32, j: i32) -> [Vec2; 4] {
        let base = self.world_from_ij(i, j);
        let w = self.tile.w;
        let h = self.tile.h;

        [
            Vec2::new(base.x + w * 0.5, base.y),
            Vec2::new(base.x + w, base.y + h * 0.5),
            Vec2::new(base.x + w * 0.5, base.y + h),
            Vec2::new(base.x, base.y + h * 0.5),
        ]
    }

    /// Attempt to resolve a tile index (i, j) from a world/screen-space point.
    pub fn ij_from_world(&self, p: Vec2, map_w: i32, map_h: i32) -> Option<(i32, i32)> {
        if map_w <= 0 || map_h <= 0 {
            return None;
        }

        let tw = self.tile.w;
        let dy = self.tile.h * 0.5;
        let rel = p - self.origin;

        let mut i = (rel.y / dy).floor() as i32;
        i = i.clamp(0, map_h - 1);

        let shift = if i & 1 == 1 { tw * 0.5 } else { 0.0 };
        let mut j = ((rel.x - shift) / tw).floor() as i32;
        j = j.clamp(0, map_w - 1);

        // Check the candidate tile and a small set of neighbours to account for
        // the rhombus shape (bounding box floors can land outside the diamond).
        let neighbours = [
            (i, j),
            (i, j + 1),
            (i + 1, j),
            (i + 1, j + 1),
            (i - 1, j),
            (i - 1, j + 1),
            (i, j - 1),
            (i + 1, j - 1),
        ];

        neighbours
            .into_iter()
            .filter(|&(ci, cj)| ci >= 0 && cj >= 0 && ci < map_h && cj < map_w)
            .find(|&(ci, cj)| self.point_inside_tile(p, ci, cj))
            .or_else(|| {
                if self.point_inside_tile(p, i, j) {
                    Some((i, j))
                } else {
                    None
                }
            })
    }

    fn point_inside_tile(&self, p: Vec2, i: i32, j: i32) -> bool {
        if i < 0 || j < 0 {
            return false;
        }
        let center = self.world_center(i, j);
        let half_w = self.tile.w * 0.5;
        let half_h = self.tile.h * 0.5;
        let dx = (p.x - center.x).abs() / half_w;
        let dy = (p.y - center.y).abs() / half_h;
        dx + dy <= 1.0 + f32::EPSILON
    }

    /// Total world-space size of the map, useful for centering within a viewport.
    pub fn map_world_size(&self, map_w: i32, map_h: i32) -> Vec2 {
        let extra = if map_h > 1 { self.tile.w * 0.5 } else { 0.0 };
        let width = self.tile.w * map_w as f32 + extra;
        let height = self.tile.h * ((map_h as f32 + 1.0) * 0.5);
        Vec2::new(width, height)
    }
}
