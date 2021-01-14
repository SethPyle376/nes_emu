use crate::emu::cpu::CPU;
use std::sync::{Arc, Mutex};
use crate::emu::bus::Bus;

use std::str;
use std::time::{Duration, Instant};
use tui::Terminal;
use tui::backend::TermionBackend;
use std::io::{Stdout};
use termion::raw::{IntoRawMode, RawTerminal};
use tui::widgets::{Block, Borders, Row, Cell, Table, TableState, List, ListItem};
use tui::layout::{Layout, Constraint, Direction, Rect, Corner};
use tui::style::{Style, Color, Modifier};
use tui::text::Span;

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
      frame_duration: 1_000_000 / frame_rate as u64,
      state: TableState::default(),
      last_draw: Instant::now(),
      should_quit: false
    }
  }

  pub fn draw(&mut self) {

    if self.bus.lock().unwrap().ram[0x0003] == 0xFF {
      self.should_quit = true;
    }

    let duration = Instant::now().duration_since(self.last_draw);

    if duration.as_micros() < self.frame_duration as u128 || self.should_quit == true {
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

    let cpu_lock = self.cpu.lock().unwrap();

    let mut cpu_info = vec!["null".to_string(); 10];

    cpu_info[0] = "Program Counter: 0x".to_string() + &*hex::encode([(cpu_lock.pc >> 8) as u8]).to_string() + &*hex::encode([(cpu_lock.pc & 0x00FF) as u8]);
    cpu_info[1] = "Stack Pointer: 0x".to_string() + &*hex::encode([cpu_lock.sp]);
    cpu_info[2] = "Register A: 0x".to_string() + &*hex::encode([cpu_lock.r_a]);
    cpu_info[3] = "Register X: 0x".to_string() + &*hex::encode([cpu_lock.r_x]);
    cpu_info[4] = "Register Y: 0x".to_string() + &*hex::encode([cpu_lock.r_y]);
    cpu_info[5] = "Register Status: 0x".to_string() + &*hex::encode([cpu_lock.r_status]);
    cpu_info[6] = "Memory Fetch Location: 0x".to_string() + &*hex::encode([(cpu_lock.location >> 8) as u8]).to_string() + &*hex::encode([(cpu_lock.location & 0x00FF) as u8]);
    cpu_info[7] = "Memory Relative Fetch Location: 0x".to_string() + &*hex::encode([(cpu_lock.relative_location >> 8) as u8]).to_string() + &*hex::encode([(cpu_lock.relative_location & 0x00FF) as u8]);
    cpu_info[8] = "Cycles: ".to_string() + &*cpu_lock.cycles.to_string();
    cpu_info[9] = "Skip Cycles: ".to_string() + &*cpu_lock.skip_cycles.to_string();

    self.terminal.draw(|f| {
      let rects = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage(33), Constraint::Percentage(33)].as_ref()).direction(Direction::Horizontal).split(Rect { x: 0, y: 0, width: 150, height: 15});
      let selected_style = Style::default().add_modifier(Modifier::REVERSED);
      let normal_style = Style::default().bg(Color::Blue);
      let header_cells = ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "0A", "0B", "0C", "0D", "0E", "0F"].iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
      let header = Row::new(header_cells).style(normal_style).height(1);
      let rows = byte_grid.iter().map(|row| {
        let cells = row.iter().map(|r| Cell::from(r.as_str()));
        Row::new(cells).height(1)
      });
      let column_widths = [Constraint::Length(2); 16];
      let t = Table::new(rows).header(header).block(Block::default().borders(Borders::ALL).title("Zero Page RAM")).highlight_style(selected_style).highlight_symbol(">> ").widths(&column_widths);
      f.render_widget(t, rects[0]);

      let cpu_list_items: Vec<ListItem> = cpu_info.iter().map(|item| {
        ListItem::new(item.as_str())
      }).collect();

      let cpu_list = List::new(cpu_list_items).block(Block::default().borders(Borders::ALL).title("CPU Status")).start_corner(Corner::TopRight);

      f.render_widget(cpu_list, rects[1]);
    }).unwrap();
  }

  pub fn cleanup(&self) {
    print!("{}[2J", 27 as char);
  }
}