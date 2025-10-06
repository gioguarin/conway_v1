use anyhow::Result;
use ratatui::{
  Terminal,
  buffer::Buffer,
  crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll, read},
  layout::{Rect, Size},
  prelude::CrosstermBackend,
  style::{Color, Stylize},
  text::{Line, Span, Text},
  widgets::Widget,
};
use std::{
  io::Stdout,
  ops::{ControlFlow, Index, IndexMut},
  thread::sleep,
  time::{Duration, Instant},
};

fn main() {
  let mut term = ratatui::init();
  let mut state = State::new(term.size().unwrap());

  let result = state.run(&mut term);

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

  fn run(&mut self, term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
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
          accumulator -= tick_rate;
        }
      }

      term.draw(|frame| frame.render_widget(&*self, frame.area()))?;

      let elapsed = last_frame.elapsed();
      if elapsed < frame_rate {
        sleep(frame_rate - elapsed);
      }

      self.frame_time = last_frame.elapsed();
    })
  }

  fn handle_events(&mut self) -> Result<ControlFlow<()>> {
    let (c_col, c_row) = (&mut self.cursor.col, &mut self.cursor.row);
    let (max_cols, max_rows) = (self.grid.cols(), self.grid.rows());
    Ok(ControlFlow::Continue(while poll(Duration::default())? {
      let event = read()?;
      if let Event::Resize(cols, rows) = event {
        self.grid.resize(rows, cols)
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

            let nr = (r as isize)
              .checked_add(dr)
              .and_then(|v| usize::try_from(v).ok());
            let nc = (c as isize)
              .checked_add(dc)
              .and_then(|v| usize::try_from(v).ok());

            if let (Some(nr), Some(nc)) = (nr, nc) {
              if let Some(cell) = self.grid.get((nr, nc)) {
                if *cell {
                  neighbors += 1;
                }
              }
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
}

impl Widget for &State {
  fn render(self, area: Rect, buf: &mut Buffer) {
    (0..self.grid.rows())
      .map(|r| {
        (0..self.grid.cols())
          .map(|c| match self.grid[(r, c)] {
            true if self.cursor.at(r, c) => Span::from("*").bg(Color::White).fg(Color::Black),
            false if self.cursor.at(r, c) => Span::from("*"),
            true => Span::from(" ").bg(Color::White),
            false => Span::from(" "),
          })
          .collect::<Line>()
      })
      .collect::<Text>()
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

  fn get(&self, (row, col): (usize, usize)) -> Option<&bool> {
    self.data.get(row * self.cols + col)
  }

  fn resize(&mut self, rows: u16, cols: u16) {
    self.data.resize(rows.into(), false);
    self.cols = cols.into();
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
