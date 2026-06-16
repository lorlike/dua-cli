use anyhow::Result;
use byte_unit::Byte;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Row, Table, TableState};
use std::collections::HashSet;
use std::path::PathBuf;

use crate::{Item, aggregate, get_path_list};

pub struct App {
    pub current_dir: PathBuf,
    pub items: Vec<Item>,
    pub history: Vec<PathBuf>,
    pub table_state: TableState,
    pub marked: HashSet<usize>,
    pub total_size: u64,
}

impl App {
    pub fn new(paths: Vec<PathBuf>) -> Result<Self> {
        let current_dir = if paths.len() == 1 && paths[0].is_dir() {
            paths[0].clone()
        } else {
            PathBuf::from(".")
        };

        let items = load_items(&current_dir)?;
        let total_size = items.iter().map(|i| i.size).sum();

        let mut table_state = TableState::default();
        table_state.select_first();

        Ok(App {
            current_dir,
            items,
            history: Vec::new(),
            table_state,
            marked: HashSet::new(),
            total_size,
        })
    }

    pub fn navigate_into(&mut self) -> Result<()> {
        let selected = self.table_state.selected();
        if let Some(idx) = selected {
            if idx < self.items.len() && self.items[idx].is_dir {
                let child_dir = self.current_dir.join(&self.items[idx].name);
                self.history.push(self.current_dir.clone());
                self.current_dir = child_dir;
                self.refresh()?;
            }
        }
        Ok(())
    }

    pub fn navigate_back(&mut self) -> Result<()> {
        if let Some(parent) = self.history.pop() {
            self.current_dir = parent;
            self.refresh()?;
        }
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.items = load_items(&self.current_dir)?;
        self.total_size = self.items.iter().map(|i| i.size).sum();
        self.marked.clear();
        self.table_state.select_first();
        Ok(())
    }

    pub fn toggle_mark(&mut self) {
        let selected = self.table_state.selected();
        if let Some(idx) = selected {
            if idx < self.items.len() {
                if self.marked.contains(&idx) {
                    self.marked.remove(&idx);
                } else {
                    self.marked.insert(idx);
                }
            }
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .spacing(1);

        let [top, main, bottom] = layout.areas(frame.area());

        // Title bar: current directory path
        let title = Line::from_iter([
            Span::from("dua ").bold().fg(Color::Green),
            Span::from(self.current_dir.to_string_lossy().to_string()).fg(Color::Yellow),
        ]);
        frame.render_widget(title.centered(), top);

        // Footer: keybinding hints
        let footer = Line::from_iter([
            Span::from(" q:quit "),
            Span::from(" j/k:↑↓ "),
            Span::from(" l:enter "),
            Span::from(" h:back "),
            Span::from(" space:mark "),
        ])
        .fg(Color::DarkGray);
        let footer_block = Block::bordered().title(footer.right_aligned());
        frame.render_widget(footer_block, bottom);

        // Table
        self.render_table(frame, main);
    }

    fn render_table(&self, frame: &mut Frame, area: Rect) {
        let header = Row::new(["Size", "%", "Filename"])
            .style(Style::new().bold())
            .bottom_margin(1);

        let rows: Vec<Row> = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let size_str = format_size(item.size);
                let pct = if self.total_size > 0 {
                    format!("{:.1}%", item.size as f64 / self.total_size as f64 * 100.0)
                } else {
                    "-".to_string()
                };

                let name_style = if self.marked.contains(&idx) {
                    Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else if item.is_dir {
                    Style::new().fg(Color::Cyan)
                } else {
                    Style::new().fg(Color::White)
                };

                let mark_prefix = if self.marked.contains(&idx) {
                    "✓ "
                } else {
                    "  "
                };

                Row::new([
                    Cell::from(size_str).style(Style::new().fg(Color::Green)),
                    Cell::from(pct).style(Style::new()),
                    Cell::from(format!("{}{}", mark_prefix, item.name)).style(name_style),
                ])
            })
            .collect();

        let widths = [
            Constraint::Percentage(25),
            Constraint::Percentage(10),
            Constraint::Percentage(65),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .column_spacing(1)
            .style(Color::White)
            .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED).bold());

        frame.render_stateful_widget(table, area, &mut self.table_state.clone());
    }
}

fn load_items(dir: &PathBuf) -> Result<Vec<Item>> {
    let path_list = get_path_list(&dir.to_string_lossy())?;
    aggregate(path_list)
}

fn format_size(bytes: u64) -> String {
    let byte = Byte::from_u64(bytes).get_appropriate_unit(byte_unit::UnitType::Binary);
    format!("{byte:.2}")
}
