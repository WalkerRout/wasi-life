use std::io::{self, Stdout};

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub trait Canvas {
  type Colour;
  fn draw_pixel(&mut self, i: usize, j: usize, colour: Self::Colour);
  fn render(&self);
}

pub struct ConsoleCanvas {
  width: usize,
  height: usize,
  grid: Vec<u8>,
  stdout: Stdout,
}

impl ConsoleCanvas {
  pub fn new(width: usize, height: usize) -> Self {
    Self {
      width,
      height,
      grid: vec![OFF_COLOUR; width * height],
      stdout: io::stdout(),
    }
  }
}

const ON_COLOUR: u8 = 1; // on-cell pixel color
const OFF_COLOUR: u8 = 0; // off-cell pixel color

impl Canvas for ConsoleCanvas {
  type Colour = u8;

  #[inline]
  fn draw_pixel(&mut self, i: usize, j: usize, colour: Self::Colour) {
    self.grid[i * self.width + j] = colour;
  }

  fn render(&self) {
    use std::io::Write;
    let lock = self.stdout.lock();
    let mut buf = std::io::BufWriter::new(lock);
    for i in 0..self.height {
      for j in 0..self.width {
        let repr = match self.grid[i * self.width + j] & 0x1 {
          ON_COLOUR => b" @ ",
          OFF_COLOUR => b" . ",
          _ => unreachable!(),
        };
        buf.write(repr).unwrap();
      }
      buf.write(b"\n").unwrap();
    }
  }
}

pub trait ProductSingletonCandidate<F, S> {
  const FST: F;
  const SND: S;
}

impl ProductSingletonCandidate<Self, Self> for u8 {
  const FST: Self = ON_COLOUR;
  const SND: Self = OFF_COLOUR;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeighbourCount(u8);

impl NeighbourCount {
  pub const MAX: u8 = 8;
  pub const MIN: u8 = 0;

  #[inline]
  pub fn get(self) -> u8 {
    self.0
  }
}

impl TryFrom<u8> for NeighbourCount {
  type Error = String;

  #[inline]
  fn try_from(byte: u8) -> Result<Self, Self::Error> {
    match byte {
      Self::MIN..=Self::MAX => Ok(Self(byte)),
      _ => Err(String::from("byte out of range for neighbour count")),
    }
  }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Cell(u8);

impl Cell {
  pub const MAX: u8 = 0b00011111;
  pub const MIN: u8 = 0;

  #[inline]
  pub fn is_alive(&self) -> bool {
    // state is not 0
    (self.0 & 0x1) != 0
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.0 == 0
  }

  #[inline]
  pub fn set_alive(&mut self) {
    self.0 |= 0x1;
  }

  #[inline]
  pub fn set_dead(&mut self) {
    self.0 &= !0x1;
  }

  #[inline]
  pub fn neighbours(&self) -> NeighbourCount {
    let count = (self.0 & 0x1e) >> 1;
    NeighbourCount::try_from(count).unwrap()
  }

  #[inline]
  pub fn try_increment(&mut self) -> bool {
    let neighbour_count = self.neighbours().get();
    if neighbour_count < NeighbourCount::MAX {
      *self = Self((self.0 & 0xe1) | ((neighbour_count + 1) << 1));
      true
    } else {
      false
    }
  }

  #[inline]
  pub fn try_decrement(&mut self) -> bool {
    let neighbour_count = self.neighbours().get();
    if neighbour_count < NeighbourCount::MAX {
      *self = Self((self.0 & 0xe1) | ((neighbour_count - 1) << 1));
      true
    } else {
      false
    }
  }
}

impl TryFrom<u8> for Cell {
  type Error = String;

  #[inline]
  fn try_from(byte: u8) -> Result<Self, Self::Error> {
    match byte {
      Self::MIN..=Self::MAX => Ok(Self(byte)),
      _ => Err(String::from("byte out of range for cell")),
    }
  }
}

pub struct World {
  cells: Vec<Cell>,
  temp_cells: Vec<Cell>,
  width: usize,
  height: usize,
}

impl World {
  pub fn new(width: usize, height: usize) -> Self {
    let cell_count = width * height;
    Self {
      cells: vec![Cell::default(); cell_count],
      temp_cells: vec![Cell::default(); cell_count],
      width,
      height,
    }
  }

  pub fn random<R: Rng>(width: usize, height: usize, rng: &mut R) -> Self {
    let mut world = World::new(width, height);
    let init_length = (world.height * world.width) / 2;
    for _ in 0..init_length {
      let i = rng.gen_range(0..world.height);
      let j = rng.gen_range(0..world.width);
      if world.cell_state(i, j) == 0 {
        world.set_cell(i, j);
      }
    }
    world
  }

  pub fn next_generation<Co, Ca>(&mut self, canvas: &mut Ca)
  where
    Co: ProductSingletonCandidate<Co, Co>,
    Ca: Canvas<Colour = Co>,
  {
    self.temp_cells.copy_from_slice(&self.cells);
    for i in 0..self.height {
      let mut j = 0;
      while j < self.width {
        let curr_cell = self.temp_cells[i * self.width + j];
        // skim past off cells with no neighbours
        if curr_cell.is_empty() {
          j += 1;
          continue;
        }
        let count = curr_cell.neighbours().get();
        if curr_cell.is_alive() {
          // cell active; turn off if doesnt have 2 or 3 neighbours
          if count != 2 && count != 3 {
            self.clear_cell(i, j);
            canvas.draw_pixel(i, j, Co::SND);
          }
        } else if count == 3 {
          // cell inactive; turn on if has exactly 3 neighbours
          self.set_cell(i, j);
          canvas.draw_pixel(i, j, Co::FST);
        }
        j += 1;
      }
    }
  }

  fn set_cell(&mut self, i: usize, j: usize) {
    let w = self.width;
    let cell_ptr = i * w + j;
    // cell is alive
    self.cells[cell_ptr].set_alive();
    for &i_offset in &[-1, 0, 1] {
      for &j_offset in &[-1, 0, 1] {
        // skip self
        if i_offset == 0 && j_offset == 0 {
          continue;
        }
        // update neighbours
        if let Some((i, j)) = self.is_valid_position(i as isize + i_offset, j as isize + j_offset) {
          self.cells[i * w + j].try_increment();
        }
      }
    }
  }

  fn clear_cell(&mut self, i: usize, j: usize) {
    let w = self.width;
    let cell_ptr = i * w + j;
    // cell is dead
    self.cells[cell_ptr].set_dead();
    for &i_offset in &[-1, 0, 1] {
      for &j_offset in &[-1, 0, 1] {
        // skip self
        if i_offset == 0 && j_offset == 0 {
          continue;
        }
        // update neighbours
        if let Some((i, j)) = self.is_valid_position(i as isize + i_offset, j as isize + j_offset) {
          self.cells[i * w + j].try_decrement();
        }
      }
    }
  }

  #[inline]
  fn cell_state(&self, i: usize, j: usize) -> u8 {
    self.cells[i * self.width + j].is_alive() as u8
  }

  #[inline]
  fn is_valid_position(&self, neighbour_i: isize, neighbour_j: isize) -> Option<(usize, usize)> {
    if neighbour_i < 0
      || neighbour_i >= self.height as isize
      || neighbour_j < 0
      || neighbour_j >= self.width as isize
    {
      None
    } else {
      Some((neighbour_i as usize, neighbour_j as usize))
    }
  }
}

pub fn main() {
  let (width, height) = (96, 96);
  let mut current_map = {
    let seed = rand::random::<u64>();
    let mut rng = StdRng::seed_from_u64(seed);
    World::random(width, height, &mut rng)
  };
  let mut canvas = ConsoleCanvas::new(width, height);
  let mut generation: u64 = 0;
  let render = false;
  loop {
    generation += 1;
    current_map.next_generation(&mut canvas);
    if render {
      print!("\x1B[2J\x1B[1;1H");
      println!("Generation: {}", generation);
      canvas.render();
    }
    if generation > 50 {
      break;
    }
  }
  println!("Total generations: {}", generation);
}
