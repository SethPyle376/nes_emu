use crate::emu::cpu::CPU;
use std::sync::{Arc, Mutex};
use crate::emu::bus::Bus;

use std::str;
use std::time::{Duration, Instant};
use tui::Terminal;
use tui::backend::TermionBackend;
use std::io::{Stdout};
use termion::raw::{IntoRawMode, RawTerminal};
use tui::widgets::{Block, Borders, Row, Cell, Table, TableState};
use tui::layout::{Layout, Constraint};
use tui::style::{Style, Color, Modifier};

pub struct DebugInterface {
  cpu: Arc<Mutex<CPU>>,
  bus: Arc<Mutex<Bus>>,
  terminal: Terminal<TermionBackend<RawTerminal<Stdout>>>,
  state: TableState,
  frame_duration: u64,
  last_draw: Instant,
  pub should_quit: bool
}

impl DebugInterface {
  pub fn new(cpu: &Arc<Mutex<CPU>>, frame_rate: u32) -> DebugInterface {
    let stdout = std::io::stdout().into_raw_mode().unwrap();
    let backend = TermionBackend::new(stdout);
    let terminal = Terminal::new(backend).unwrap();

    // clear the screen
    print!("{}[2J", 27 as char);

    DebugInterface {
      cpu: cpu.clone(),
      bus: Arc::clone(&cpu.lock().unwrap().bus),
      terminal,
      frame_duration: 1000 / frame_rate as u64,
      state: TableState::default(),
      last_draw: Instant::now(),
      should_quit: false
    }
  }

  pub fn draw(&mut self) {
    let duration = Instant::now().duration_since(self.last_draw);

    if duration.as_millis() < 1000 || self.should_quit == true {
      return;
    }

    self.last_draw = Instant::now();

    let lock = self.bus.lock().unwrap();
    let ram = lock.ram.to_vec();

    let ram_hex = hex::encode(&ram);

    let formatted = ram_hex.as_bytes().chunks(2).map(str::from_utf8).collect::<Result<Vec<&str>, _>>().unwrap();

    let mut byte_grid = vec![vec!["00".to_string(); 16]; 16];

    for y in 0..16 {
      for x in 0..16 {
        byte_grid[y][x] = formatted[y * 16 + x].to_owned();
      }
    }

    self.terminal.draw(|f| {
      let rects = Layout::default().constraints([Constraint::Percentage(100)].as_ref()).split(f.size());
      let selected_style = Style::default().add_modifier(Modifier::REVERSED);
      let normal_style = Style::default().bg(Color::Blue);
      let header_cells = ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "0A", "0B", "0C", "0D", "0E", "0F"].iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
      let header = Row::new(header_cells).style(normal_style).height(1).bottom_margin(1);
      let rows = byte_grid.iter().map(|row| {
        let cells = row.iter().map(|r| Cell::from(r.as_str()));
        Row::new(cells).height(1).bottom_margin(1)
      });
      let column_widths = [Constraint::Length(5); 16];
      let t = Table::new(rows).header(header).block(Block::default().borders(Borders::ALL).title("Zero Page RAM")).highlight_style(selected_style).highlight_symbol(">> ").widths(&column_widths);
      f.render_widget(t, rects[0]);
    }).unwrap();
  }
}