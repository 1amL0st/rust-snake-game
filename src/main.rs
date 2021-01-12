use std::{intrinsics::transmute, time::Duration};

use app::AppExit;
use bevy::prelude::*;
use bevy::{app};

use rand::*;

mod metrics;
use metrics::{Metrics, compute_metrics};

mod types;
use types::{Position, Direction, Size};

/*
  cargo +nightly run  --features bevy/dynamic
  cargo +nightly run
*/

struct Fruit;
struct SnakeHead;
struct SnakeSegment;
#[derive(Default)]
struct SnakeSegments(Vec<Entity>);
struct SnakeMoveTimer(Timer);

struct SnakeGrowthEvent;
#[derive(Default)]
struct SnakeTailLastPos(Option<Position>);

struct GameOverEvent(String);

struct Materials {
  snake_head: Handle<ColorMaterial>,
  fruit: Handle<ColorMaterial>,
  snake_segment: Handle<ColorMaterial>,
  board_cell: Handle<ColorMaterial>
}

fn game_startup(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
  commands.spawn(Camera2dBundle::default());
  commands.insert_resource(Materials {
    snake_head: materials.add(Color::rgb(0.0, 0.8, 0.0).into()),
    fruit: materials.add(Color::rgb(1., 0.0, 0.0).into()),
    snake_segment: materials.add(Color::rgb(0.7, 0.7, 0.3).into()),
    board_cell: materials.add(Color::rgb(0.0, 0.0, 0.3).into())
  });
}

fn spawn_segment(
  commands: &mut Commands,
  material: &Handle<ColorMaterial>,
  position: Position,
  size: Size
) -> Entity {
  commands.spawn(SpriteBundle {
    material: material.clone(),
    ..Default::default()
  })
  .with(SnakeSegment)
  .with(position)
  .with(size)
  .current_entity()
  .unwrap()
}

fn spawn_snake(
  commands: &mut Commands,
  materials: Res<Materials>,
  metrics: Res<Metrics>,
  mut snake_segments: ResMut<SnakeSegments>
) {
  snake_segments.0 = vec![
    commands.spawn(SpriteBundle {
      material: materials.snake_head.clone(),
      transform: Transform::from_translation(Vec3::new(0., 0., 10.)),
      ..Default::default()
    })
    .with(Position::new(metrics.board_size as i32 / 2, metrics.board_size as i32 / 2))
    .with(Size::new(metrics.snake_head_size, metrics.snake_head_size))
    .with(SnakeHead)
    .with(Direction::Down)
    .with(SnakeSegment)
    .current_entity().unwrap(),
  
    // spawn_segment(
    //   commands,
    //   &materials.snake_segment,
    //   Position::new(metrics.board_size as i32 / 2, metrics.board_size as i32 / 2 + 1),
    //   Size::new(metrics.snake_segment_size, metrics.snake_segment_size)
    // ),

    // spawn_segment(
    //   commands,
    //   &materials.snake_segment,
    //   Position::new(metrics.board_size as i32 / 2, metrics.board_size as i32 / 2 + 2),
    //   Size::new(metrics.snake_segment_size, metrics.snake_segment_size)
    // )
  ];
}

fn spawn_board(
  commands: &mut Commands,
  materials: Res<Materials>,
  metrics: Res<Metrics>
) {
  for y in 0..metrics.board_size {
    for x in 0..metrics.board_size {
      commands.spawn(SpriteBundle {
        material: materials.board_cell.clone(),
        ..Default::default()
      })
      .with(Position::new(x, y))
      .with(Size::new(
        metrics.board_cell_size - metrics.border_size,
        metrics.board_cell_size - metrics.border_size
      ));
    }
  }
}

fn get_fruit_pos(
  mut fruits: Query<(&mut Fruit, &mut Transform, Entity)>,
  snake_segments: Query<&Position, With<SnakeSegment>>,
  metrics: &Res<Metrics>
) -> Option<Position> {

  let mut positions: Vec<Position> = Vec::with_capacity(metrics.board_size.pow(2) as usize);
  for y in 0..metrics.board_size {
    for x in 0..metrics.board_size {
      positions.push(Position::new(x, y));
    }
  }

  positions = positions.into_iter()
    .filter(|pos| snake_segments.iter().any(|p| pos == p) == false)
    .collect();

  if positions.len() != 0 {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0, positions.len());

    Some(Position::new(
      positions[index].x,
      positions[index].y
    ))
  } else {
    None
  }
}

fn spawn_fruit(
  commands: &mut Commands,
  materials: Res<Materials>,
  mut fruits: Query<(&mut Fruit, &mut Transform, Entity)>,
  snake_segments: Query<&Position, With<SnakeSegment>>,
  metrics: Res<Metrics>
) {
  let iterator = fruits.iter_mut();

  if iterator.len() == 0 {
    if let Some(pos) = get_fruit_pos(fruits, snake_segments, &metrics) {
      commands.spawn(SpriteBundle {
        material: materials.fruit.clone(),
        ..Default::default()
      })
      .with(Fruit)
      .with(Position::new(pos.x as i32, pos.y as i32))
      .with(Size::new(metrics.fruit_size, metrics.fruit_size));
    }
  }
}

fn snake_timer(time: Res<Time>, mut snake_timer: ResMut<SnakeMoveTimer>) {
  snake_timer.0.tick(time.delta_seconds());
}

fn move_snake_forward(
  snake_segments: ResMut<SnakeSegments>,
  mut positions: Query<&mut Position>,
  directions: Query<&Direction>,
  timer: ResMut<SnakeMoveTimer>,
  mut tail_last_pos: ResMut<SnakeTailLastPos>
) {
  if !timer.0.finished() {
    return;
  }

  let mut segments = snake_segments.0.iter();

  let head = *segments.next().unwrap();
  let mut head_pos = positions.get_mut(head).unwrap();
  let mut old_pos = *head_pos;
  match directions.get(head).unwrap() {
    Direction::Down => head_pos.y -= 1,
    Direction::Up => head_pos.y += 1,
    Direction::Left => head_pos.x -= 1,
    Direction::Right => head_pos.x += 1
  }

  for segment in segments {
    let mut seg_pos = positions.get_mut(*segment).unwrap();
    let pos = *seg_pos;
    *seg_pos = old_pos;
    old_pos = pos;
  }

  *tail_last_pos = SnakeTailLastPos(Some(old_pos));
}

fn handle_keyboard_snake_direction(
  keyboard_input: Res<Input<KeyCode>>,
  mut snake_head: Query<(Entity, &mut SnakeHead)>,
  mut directions: Query<&mut Direction>
) {
  for (entity, _head) in snake_head.iter_mut() {
    let mut direction = directions.get_mut(entity).unwrap();
    *direction = 
    if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
      Direction::Left
    }
    else if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
      Direction::Right
    }
    else if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
      Direction::Down
    }
    else if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
      Direction::Up
    } else {
      *direction
    };
  }
}

fn handle_keyboard(
  mut events: ResMut<app::Events<app::AppExit>>,
  keyboard_input: Res<Input<KeyCode>>
) {
  if keyboard_input.pressed(KeyCode::Escape) {
    events.send(app::AppExit);
  }
}

fn map_world_to_screen(
  mut sprites: Query<(&mut Transform, &mut Sprite, &Position, &Size)>,
  metrics: Res<Metrics>
) {
  let start_x = -metrics.window_size / 2. + metrics.board_cell_size / 2.;
  let start_y = -metrics.window_size / 2. + metrics.board_cell_size / 2.;

  for (mut transform, mut sprite, pos, size) in sprites.iter_mut() {
    transform.translation.x = (pos.x as f32) * metrics.board_cell_size + start_x;
    transform.translation.y = (pos.y as f32) * metrics.board_cell_size + start_y;
    sprite.size = Vec2::new(size.width, size.height);
  }
}

fn snake_eating(
  commands: &mut Commands,
  fruits: Query<(&Position, Entity, &Fruit)>,
  timer: Res<SnakeMoveTimer>,
  snake_head: Query<(&SnakeHead, &Position)>,
  mut growth_events: ResMut<Events<SnakeGrowthEvent>>
) {
  if !timer.0.finished() {
    return;
  }

  let head_pos = snake_head.iter().next().unwrap().1;
  let (fruit_pos, entity, _) = fruits.iter().next().unwrap();

  if *head_pos == *fruit_pos {
    commands.despawn(entity);
    growth_events.send(SnakeGrowthEvent);
  }
}

fn snake_growth_event_handler(
  commands: &mut Commands,
  materials: Res<Materials>,
  metrics: Res<Metrics>,
  last_tail_pos: Res<SnakeTailLastPos>,
  mut segments: ResMut<SnakeSegments>,
  mut growth_reader: Local<EventReader<SnakeGrowthEvent>>,
  growth_events: Res<Events<SnakeGrowthEvent>>
) {
  if growth_reader.iter(&growth_events).next().is_some() {
    segments.0.push(
      spawn_segment(
        commands,
        &materials.snake_segment,
        last_tail_pos.0.unwrap(),
        Size::new(metrics.snake_segment_size, metrics.snake_segment_size)
      )
    )
  }
}

fn is_game_over(
  mut game_over_events: ResMut<Events<GameOverEvent>>,
  segments: Query<&Position, With<SnakeSegment>>,
  metrics: Res<Metrics>,
  timer: Res<SnakeMoveTimer>
) {
  let mut iterator = segments.iter();
  if segments.iter().count() == metrics.board_size.pow(2) as usize {
    println!("You won this game!");
    game_over_events.send(GameOverEvent(String::from("You won!")));
  }
  
  if !timer.0.finished() {
    return;
  }

  let head_pos = iterator.next().unwrap();
  if head_pos.x < 0 || head_pos.x >= metrics.board_size
    || head_pos.y < 0 || head_pos.y >= metrics.board_size {
      game_over_events.send(GameOverEvent(String::from("Snake is crawled away!")));
      return;
  }

  if iterator.find(|seg_pos| *seg_pos == head_pos).is_some() {
    game_over_events.send(GameOverEvent(String::from("Snake ate itself!")));
  }
}

fn game_over_event_handler(
  mut app_exit_events: ResMut<Events<AppExit>>,
  mut game_over_reader: Local<EventReader<GameOverEvent>>,
  game_over_events: Res<Events<GameOverEvent>>
) {
  if let Some(event) = game_over_reader.iter(&game_over_events).next(){
    println!("Game over: {}", event.0);
    app_exit_events.send(AppExit);
  }
}

fn main() {
  App::build()
  .add_resource(WindowDescriptor {
    title: String::from("Snake game"),
    width: 720.,
    height: 720.,
    resizable: false,
    ..Default::default()
  })
  .add_resource(ClearColor(Color::rgb(0., 0., 0.4)))
  .add_resource(SnakeSegments::default())
  .add_resource(Metrics::empty())
  .add_resource(SnakeTailLastPos::default())
  .add_startup_system(game_startup.system())
  .add_startup_stage("Compute metrics", SystemStage::single(compute_metrics.system()))
  .add_startup_stage("Snake spawn", SystemStage::single(spawn_snake.system()))
  .add_startup_stage("Board spawn", SystemStage::single(spawn_board.system()))
  .add_resource(SnakeMoveTimer(Timer::new(Duration::from_millis(500u64), true)))
  .add_system(is_game_over.system())
  .add_system(spawn_fruit.system())
  .add_system(map_world_to_screen.system())
  .add_system(move_snake_forward.system())
  .add_system(snake_eating.system())
  .add_system(handle_keyboard_snake_direction.system())
  .add_system(handle_keyboard.system())
  .add_system(snake_timer.system())
  .add_system(snake_growth_event_handler.system())
  .add_system(game_over_event_handler.system())
  .add_event::<GameOverEvent>()
  .add_event::<SnakeGrowthEvent>()
  .add_plugins(DefaultPlugins)
  .run()
}