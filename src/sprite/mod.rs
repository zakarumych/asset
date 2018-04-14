
pub mod anim;

/// Sprite sheets are multiple sprites embedded into single texture object commonly aligned into grid.
/// They are primarily used in for sprite animations
/// where individual sprites represent frames of animations. Typically row per animation.
/// The second common use is for tile sets where individual sprites represent variety of tiles in set.
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
pub struct SpriteSheet {
    /// Columns count of the grid. Number of frames in one row.
    columns: u32,
    /// Rows count of the grid. Number of frames in one column.
    rows: u32,
}

/// Rectangular to get sprite from texture.
#[repr(C)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl SpriteSheet {
    /// Get frame rect by index
    pub fn sample(&self, frame: u32) -> Rect {
        assert_ne!(0, self.rows);
        assert_ne!(0, self.columns);
        let column = frame % self.columns;
        let row = frame / self.columns;
        assert!(row < self.rows);
        let w = 1f32 / self.columns as f32;
        let x = w * column as f32;
        let h = 1f32 / self.rows as f32;
        let y = h * row as f32;
        Rect {
            x, y, w, h,
        }
    }
}
