/// Packed occupancy / nutrient cell. `0` is empty / depleted.
pub type Cell = u8;

/// Row-major dense field. Indexing is `y * width + x`.
#[derive(Clone, Debug)]
pub struct Grid {
    width: u32,
    height: u32,
    data: Vec<Cell>,
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Self {
        let len = (width as usize)
            .checked_mul(height as usize)
            .expect("grid dimensions overflow");
        Self {
            width,
            height,
            data: vec![0; len],
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn as_slice(&self) -> &[Cell] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [Cell] {
        &mut self.data
    }

    #[inline]
    pub fn index(&self, x: u32, y: u32) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height
    }

    #[inline]
    pub fn get(&self, x: u32, y: u32) -> Cell {
        self.data[self.index(x, y)]
    }

    #[inline]
    pub fn set(&mut self, x: u32, y: u32, value: Cell) {
        let i = self.index(x, y);
        self.data[i] = value;
    }

    pub fn filled_count(&self) -> usize {
        self.data.iter().filter(|&&c| c > 0).count()
    }

    pub fn sum_u64(&self) -> u64 {
        self.data.iter().map(|&c| c as u64).sum()
    }
}
