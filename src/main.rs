use anyhow::Result;
use rand::Rng;
use ratatui::{
  Terminal,
  buffer::Buffer,
  crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll, read},
  layout::{Constraint, Direction, Layout, Rect, Size},
  prelude::CrosstermBackend,
  style::{Color, Style},
  symbols::Marker,
  text::{Line, Span},
  widgets::{Widget, canvas::{Canvas, Circle, Points}, Paragraph},
};
use std::{
  env::args,
  io::Stdout,
  ops::{ControlFlow, Index, IndexMut},
  thread::sleep,
  time::{Duration, Instant},
};

const HELP: &str = "-help-
controls:
  move cursor: arrow keys
  activate cell: spacebar
  spawn random pattern: r
  change speed:
    slower = [
    faster = ]
  pause: p
  clear grid: x
flags:
  -r/--random: enable random activations
control + c to exit";

fn main() {
  let mut random = false;
  if let Some(arg) = args().nth(1) {
    match arg.as_str() {
      "-h" | "--help" => return println!("{}", HELP),
      "-r" | "--random" => random = true,
      _ => {}
    }
  }

  let mut term = ratatui::init();
  let mut state = State::new(term.size().unwrap());

  if random {
    state.paused = false
  }

  let result = state.run(&mut term, random);
  ratatui::restore();

  if let Err(e) = result {
    eprintln!("Error: {}", e);
  }
}

struct State {
  grid: Grid,
  cursor: Cursor,
  tick_rate: TickRate,
  frame_time: Duration,
  paused: bool,
}

impl State {
  fn new(term_size: Size) -> Self {
    Self {
      grid: Grid::new(term_size),
      cursor: Cursor::new(term_size),
      tick_rate: TickRate::Normal,
      frame_time: Duration::ZERO,
      paused: true,
    }
  }

  fn clear(&mut self) {
    for i in 0..self.grid.data.len() {
      self.grid.data[i] = false;
    }
  }

  fn spawn_pattern_at_cursor(&mut self) {
    let mut rng = rand::thread_rng();

    let pattern = match rng.gen_range(0..6) {
      0 => Pattern::Glider,
      1 => Pattern::Blinker,
      2 => Pattern::Toad,
      3 => Pattern::Beacon,
      4 => Pattern::Pulsar,
      _ => Pattern::LightweightSpaceship,
    };

    // Place pattern centered at cursor position
    self.place_pattern(pattern, self.cursor.row, self.cursor.col);
  }

  fn run(&mut self, term: &mut Terminal<CrosstermBackend<Stdout>>, random: bool) -> Result<()> {
    let frame_rate = Duration::from_secs_f64(1. / 60.);
    let mut accumulator = Duration::ZERO;
    let mut last_frame = Instant::now();

    Ok(loop {
      if self.handle_events()?.is_break() {
        break;
      }

      let tick_rate: Duration = self.tick_rate.into();
      let delta = last_frame.elapsed();
      last_frame = Instant::now();

      if !self.paused {
        accumulator += delta;
        while accumulator >= tick_rate {
          self.update();
          if random {
            self.spawn_random_pattern();
          }
          accumulator -= tick_rate;
        }
      }

      term.draw(|frame| {
        let chunks = Layout::default()
          .direction(Direction::Vertical)
          .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
          ])
          .split(frame.area());

        frame.render_widget(&*self, chunks[0]);

        // Render status bar
        let status_line = Line::from(vec![
          Span::raw(" "),
          Span::styled("↑↓←→", Style::default().fg(Color::Cyan)),
          Span::raw(" Move | "),
          Span::styled("Space", Style::default().fg(Color::Green)),
          Span::raw(" Toggle | "),
          Span::styled("R", Style::default().fg(Color::Blue)),
          Span::raw(" Random | "),
          Span::styled("P", Style::default().fg(Color::Yellow)),
          Span::raw(" Pause | "),
          Span::styled("[ ]", Style::default().fg(Color::Magenta)),
          Span::raw(" Speed | "),
          Span::styled("X", Style::default().fg(Color::Red)),
          Span::raw(" Clear | "),
          Span::styled("Ctrl+C", Style::default().fg(Color::Gray)),
          Span::raw(" Exit"),
        ]);

        let status_bar = Paragraph::new(status_line)
          .style(Style::default().bg(Color::DarkGray));
        frame.render_widget(status_bar, chunks[1]);
      })?;

      let elapsed = last_frame.elapsed();
      if elapsed < frame_rate {
        sleep(frame_rate - elapsed);
      }

      self.frame_time = last_frame.elapsed();
    })
  }

  fn handle_events(&mut self) -> Result<ControlFlow<()>> {
    Ok(ControlFlow::Continue(while poll(Duration::default())? {
      let event = read()?;
      if let Event::Resize(cols, rows) = event {
        self.grid.resize(rows.into(), cols.into());
        self.cursor = Cursor::new(Size {
          width: cols,
          height: rows,
        })
      }
      if let Event::Key(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        ..
      }) = event
      {
        if (code, modifiers) == (KeyCode::Char('c'), KeyModifiers::CONTROL) {
          return Ok(ControlFlow::Break(()));
        }
        let (c_col, c_row) = (&mut self.cursor.col, &mut self.cursor.row);
        let (max_cols, max_rows) = (self.grid.cols(), self.grid.rows());
        match code {
          KeyCode::Left => *c_col = (*c_col + max_cols - 1) % max_cols,
          KeyCode::Right => *c_col = (*c_col + 1) % max_cols,
          KeyCode::Up => *c_row = (*c_row + max_rows - 1) % max_rows,
          KeyCode::Down => *c_row = (*c_row + 1) % max_rows,
          KeyCode::Char('p') => self.paused = !self.paused,
          KeyCode::Char(']') => self.tick_rate.increase(),
          KeyCode::Char('[') => self.tick_rate.decrease(),
          KeyCode::Char(' ') => {
            let alive = &mut self.grid[(*c_row, *c_col)];
            *alive = !*alive
          }
          KeyCode::Char('x') => self.clear(),
          KeyCode::Char('r') => self.spawn_pattern_at_cursor(),
          _ => {}
        }
      }
    }))
  }

  fn update(&mut self) {
    let mut next = self.grid.clone();

    for r in 0..self.grid.rows() {
      for c in 0..self.grid.cols() {
        let mut neighbors = 0;

        for dr in [-1, 0, 1] {
          for dc in [-1, 0, 1] {
            if dr == 0 && dc == 0 {
              continue;
            }

            let nr = ((r as isize + dr).rem_euclid(self.grid.rows() as isize)) as usize;
            let nc = ((c as isize + dc).rem_euclid(self.grid.cols() as isize)) as usize;

            if self.grid[(nr, nc)] {
              neighbors += 1;
            }
          }
        }

        let alive = &mut next[(r, c)];
        *alive = match (*alive, neighbors) {
          (true, 2..=3) => true,
          (false, 3) => true,
          _ => false,
        };
      }
    }

    self.grid = next;
  }

  fn spawn_random_pattern(&mut self) {
    let mut rng = rand::thread_rng();

    if rng.gen_range(0..10) != 0 {
      return;
    }

    let pattern = match rng.gen_range(0..6) {
      0 => Pattern::Glider,
      1 => Pattern::Blinker,
      2 => Pattern::Toad,
      3 => Pattern::Beacon,
      4 => Pattern::Pulsar,
      _ => Pattern::LightweightSpaceship,
    };

    let row = rng.gen_range(0..self.grid.rows().saturating_sub(15));
    let col = rng.gen_range(0..self.grid.cols().saturating_sub(15));

    self.place_pattern(pattern, row, col);
  }

  fn place_pattern(&mut self, pattern: Pattern, start_row: usize, start_col: usize) {
    let cells = pattern.cells();

    for (dr, dc) in cells {
      let r = start_row + dr;
      let c = start_col + dc;

      if r < self.grid.rows() && c < self.grid.cols() {
        self.grid[(r, c)] = true;
      }
    }
  }
}

impl Widget for &State {
  fn render(self, area: Rect, buf: &mut Buffer) {
    // Calculate cell size based on terminal dimensions
    let cell_size = 2.0;
    let x_bounds = [0.0, self.grid.cols() as f64 * cell_size];
    let y_bounds = [0.0, self.grid.rows() as f64 * cell_size];

    Canvas::default()
      .marker(Marker::Dot)
      .x_bounds(x_bounds)
      .y_bounds(y_bounds)
      .paint(|ctx| {
        // Draw ALL cells - both alive and dead
        for r in 0..self.grid.rows() {
          for c in 0..self.grid.cols() {
            let x = c as f64 * cell_size + cell_size / 2.0;
            let y = (self.grid.rows() - 1 - r) as f64 * cell_size + cell_size / 2.0;
            let is_cursor = self.cursor.at(r, c);
            let is_alive = self.grid[(r, c)];

            if is_alive {
              // Draw filled circle for live cells
              ctx.draw(&Circle {
                x,
                y,
                radius: cell_size * 0.4,
                color: if is_cursor {
                  Color::Cyan  // Cursor on live cell
                } else {
                  Color::White  // Normal live cell
                },
              });
            } else {
              // Draw hollow circle outline for dead cells
              let radius = cell_size * 0.35;
              let num_points = 16; // Number of points to approximate circle
              let color = if is_cursor {
                Color::Yellow  // Cursor on dead cell
              } else {
                Color::DarkGray  // Normal dead cell outline
              };

              // Draw circle outline using points
              let points: Vec<(f64, f64)> = (0..num_points)
                .map(|i| {
                  let angle = 2.0 * std::f64::consts::PI * i as f64 / num_points as f64;
                  (
                    x + radius * angle.cos(),
                    y + radius * angle.sin(),
                  )
                })
                .collect();

              ctx.draw(&Points {
                coords: &points,
                color,
              });
            }
          }
        }
      })
      .render(area, buf);
  }
}

#[derive(Clone)]
struct Grid {
  data: Vec<bool>,
  cols: usize,
}

impl Grid {
  fn new(Size { width, height }: Size) -> Self {
    Self {
      data: vec![false; (height * width).into()],
      cols: width.into(),
    }
  }

  fn rows(&self) -> usize {
    self.data.len() / self.cols
  }

  fn cols(&self) -> usize {
    self.cols
  }

  fn resize(&mut self, rows: usize, cols: usize) {
    let mut data = vec![false; rows * cols];

    for r in 0..self.rows().min(rows) {
      for c in 0..self.cols().min(cols) {
        data[r * cols + c] = self.data[r * self.cols() + c];
      }
    }

    self.data = data;
    self.cols = cols;
  }
}

impl Index<(usize, usize)> for Grid {
  type Output = bool;

  fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
    &self.data[row * self.cols + col]
  }
}

impl IndexMut<(usize, usize)> for Grid {
  fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
    &mut self.data[row * self.cols + col]
  }
}

#[derive(PartialEq, Eq)]
struct Cursor {
  row: usize,
  col: usize,
}

impl Cursor {
  fn new(Size { width, height }: Size) -> Self {
    Self {
      row: (height / 2).into(),
      col: (width / 2).into(),
    }
  }

  fn at(&self, row: usize, col: usize) -> bool {
    Cursor { row, col } == *self
  }
}

#[derive(Clone, Copy)]
enum TickRate {
  Slow,
  Normal,
  Fast,
}

impl TickRate {
  fn increase(&mut self) {
    *self = match *self {
      Self::Slow => Self::Normal,
      Self::Normal => Self::Fast,
      Self::Fast => Self::Slow,
    }
  }

  fn decrease(&mut self) {
    *self = match *self {
      Self::Slow => Self::Fast,
      Self::Normal => Self::Slow,
      Self::Fast => Self::Normal,
    }
  }
}

impl From<TickRate> for Duration {
  fn from(value: TickRate) -> Self {
    Duration::from_secs_f64(match value {
      TickRate::Slow => 1.,
      TickRate::Normal => 1. / 5.,
      TickRate::Fast => 1. / 10.,
    })
  }
}

enum Pattern {
  Glider,
  Blinker,
  Toad,
  Beacon,
  Pulsar,
  LightweightSpaceship,
}

impl Pattern {
  fn cells(&self) -> Vec<(usize, usize)> {
    match self {
      Pattern::Glider => vec![(0, 1), (1, 2), (2, 0), (2, 1), (2, 2)],
      Pattern::Blinker => vec![(1, 0), (1, 1), (1, 2)],
      Pattern::Toad => vec![(1, 1), (1, 2), (1, 3), (2, 0), (2, 1), (2, 2)],
      Pattern::Beacon => vec![(0, 0), (0, 1), (1, 0), (2, 3), (3, 2), (3, 3)],
      Pattern::Pulsar => vec![
        (0, 2),
        (0, 3),
        (0, 4),
        (0, 8),
        (0, 9),
        (0, 10),
        (2, 0),
        (2, 5),
        (2, 7),
        (2, 12),
        (3, 0),
        (3, 5),
        (3, 7),
        (3, 12),
        (4, 0),
        (4, 5),
        (4, 7),
        (4, 12),
        (5, 2),
        (5, 3),
        (5, 4),
        (5, 8),
        (5, 9),
        (5, 10),
        (7, 2),
        (7, 3),
        (7, 4),
        (7, 8),
        (7, 9),
        (7, 10),
        (8, 0),
        (8, 5),
        (8, 7),
        (8, 12),
        (9, 0),
        (9, 5),
        (9, 7),
        (9, 12),
        (10, 0),
        (10, 5),
        (10, 7),
        (10, 12),
        (12, 2),
        (12, 3),
        (12, 4),
        (12, 8),
        (12, 9),
        (12, 10),
      ],
      Pattern::LightweightSpaceship => vec![
        (0, 1),
        (0, 4),
        (1, 0),
        (2, 0),
        (2, 4),
        (3, 0),
        (3, 1),
        (3, 2),
        (3, 3),
      ],
    }
  }
}
