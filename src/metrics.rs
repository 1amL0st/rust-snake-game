use bevy::prelude::*;

pub struct Metrics {
  pub board_size: i32, // In cells
  pub board_cell_size: f32,
  pub fruit_size: f32,
  pub snake_head_size: f32,
  pub snake_segment_size: f32,
  pub bordered_cell_size: f32,

  pub window_size: f32,
  pub border_size: f32,
}

impl Metrics {
  pub fn empty() -> Self {
    Metrics {
      board_size: 0,
      board_cell_size: 0.,
      snake_segment_size: 0.,
      bordered_cell_size: 0.,
      fruit_size: 0.,
      window_size: 0.,
      border_size: 0.,
      snake_head_size: 0.,
    }
  }
}

pub fn compute_metrics(mut metrics: ResMut<Metrics>, windows: Res<Windows>) {
  let window = windows.get_primary().unwrap();

  metrics.board_size = 10;
  metrics.window_size = window.width();
  metrics.border_size = 4.;

  metrics.board_cell_size = metrics.window_size / (metrics.board_size as f32);
  metrics.bordered_cell_size = metrics.board_cell_size - metrics.border_size;
  metrics.fruit_size = metrics.board_cell_size - 32.;

  metrics.snake_head_size = metrics.board_cell_size - 12.;
  metrics.snake_segment_size = metrics.board_cell_size - 24.;
}