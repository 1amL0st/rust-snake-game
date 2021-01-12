#[derive(Copy, Clone, PartialEq, Default)]
pub struct Position {
  pub x: i32,
  pub y: i32
}

impl Position {
  pub fn new(x: i32, y: i32) -> Self {
    Position {
      x,
      y
    }
  }
}

#[derive(Clone, Copy)]
pub enum Direction {
  Left,
  Right,
  Up,
  Down
}

impl Direction {
  pub fn opposite(&self) -> Self {
    match (*self) {
      Direction::Left => Direction::Right,
      Direction::Right => Direction::Left,
      Direction::Up => Direction::Down,
      Direction::Down => Direction::Up,
    }
  }
}
pub struct Size {
  pub width: f32,
  pub height: f32
}

impl Size {
  pub fn new(width: f32, height: f32) -> Self {
    Size {
      width,
      height
    }
  }
}