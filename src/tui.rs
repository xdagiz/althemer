use crate::config::AlthemerConfig;
use crate::switcher::switch_theme;
use crate::themes::{Theme, ThemeColors, list_themes};
use crossterm::{
    cursor::SetCursorStyle,
    event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
};
use ratatui::layout::Position;
use ratatui::widgets::Padding;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap,
    },
};
use std::path::{Path, PathBuf};
use std::{io, vec};

const TEXT_COLOR: Color = Color::Rgb(205, 214, 244);
const SUBTEXT_COLOR: Color = Color::Rgb(166, 173, 200);
const SURFACE_COLOR: Color = Color::Rgb(49, 50, 68);
const SELECTED_STYLE: Style = Style::new()
    .fg(TEXT_COLOR)
    .bg(SURFACE_COLOR)
    .add_modifier(Modifier::BOLD);

pub struct App {
    should_exit: bool,
    themes: ThemesList,
    status_message: Option<String>,
    custom_themes_path: Option<PathBuf>,
    input_mode: InputMode,
    filter_input: String,
    character_index: usize,
    show_preview: bool,
    quit_on_select: bool,
}

struct ThemesList {
    items: Vec<Theme>,
    filtered_indices: Vec<usize>,
    selected: usize,
    scroll: usize,
    cached_colors: Option<ThemeColors>,
}

enum InputMode {
    Normal,
    Filtering,
}

impl Default for App {
    fn default() -> Self {
        Self::new(None, &AlthemerConfig::default())
    }
}

impl App {
    pub fn new(custom_themes_path: Option<&Path>, config: &AlthemerConfig) -> Self {
        let (items, status_message) = match list_themes(custom_themes_path) {
            Ok(items) if items.is_empty() => (items, Some("No themes found".to_string())),
            Ok(items) => (items, None),
            Err(err) => (Vec::new(), Some(format!("Failed to load themes: {err}"))),
        };

        let filtered_indices = (0..items.len()).collect();
        Self {
            should_exit: false,
            themes: ThemesList {
                items,
                filtered_indices,
                selected: 0,
                scroll: 0,
                cached_colors: None,
            },
            status_message,
            custom_themes_path: custom_themes_path.map(Path::to_path_buf),
            input_mode: InputMode::Normal,
            filter_input: String::new(),
            character_index: 0,
            show_preview: config.show_preview,
            quit_on_select: config.quit_on_select,
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_exit {
            let area = Rect::from(terminal.size()?);
            terminal.draw(|frame| {
                let area = frame.area();
                frame.render_widget(&mut self, area);

                if matches!(self.input_mode, InputMode::Filtering) {
                    let main_layout =
                        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]);
                    let [_, footer_area] = area.layout(&main_layout);
                    let cursor_x = footer_area.x + 1 + self.character_index as u16;
                    let cursor_y = footer_area.y;
                    frame.set_cursor_position(Position::new(cursor_x, cursor_y));
                    let _ = execute!(std::io::stdout(), SetCursorStyle::SteadyUnderScore);
                }
            })?;

            if let Some(key) = event::read()?.as_key_press_event() {
                self.handle_key(key, area);
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent, area: Rect) {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('q') => self.should_exit = true,
                KeyCode::Char('j') | KeyCode::Down => self.select_next(area),
                KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.select_next(area)
                }
                KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.select_previous()
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.should_exit = true
                }
                KeyCode::PageDown | KeyCode::Char('d')
                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    self.page_down(area)
                }
                KeyCode::PageUp | KeyCode::Char('u')
                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    self.page_up(area)
                }
                KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                KeyCode::Char('G') | KeyCode::End => self.select_last(area),
                KeyCode::Char('/') => {
                    self.input_mode = InputMode::Filtering;
                    self.character_index = self.filter_input.chars().count();
                    self.apply_filter(area);
                }
                KeyCode::Esc => {
                    if !self.filter_input.is_empty() {
                        self.filter_input.clear();
                        self.reset_cursor();
                        self.apply_filter(area);
                    }
                }
                KeyCode::Enter => self.confirm_selection(),
                _ => {}
            },
            InputMode::Filtering if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.should_exit = true
                }
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.filter_input.clear();
                    self.reset_cursor();
                    self.apply_filter(area);
                }
                KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.delete_word(area);
                }
                KeyCode::Char(to_insert) => self.enter_char(to_insert, area),
                KeyCode::Backspace => self.delete_char(area),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.filter_input.clear();
                    self.reset_cursor();
                    self.apply_filter(area);
                }
                _ => {}
            },
            InputMode::Filtering => {}
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char, area: Rect) {
        let index = self.byte_index();
        self.filter_input.insert(index, new_char);
        self.move_cursor_right();
        self.apply_filter(area);
    }

    fn byte_index(&self) -> usize {
        self.filter_input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.filter_input.len())
    }

    fn delete_char(&mut self, area: Rect) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;
            let before = self.filter_input.chars().take(from_left_to_current_index);
            let after = self.filter_input.chars().skip(current_index);

            self.filter_input = before.chain(after).collect();
            self.move_cursor_left();
            self.apply_filter(area);
        }
    }

    fn delete_word(&mut self, area: Rect) {
        if self.character_index == 0 {
            return;
        }

        let chars: Vec<char> = self.filter_input.chars().collect();
        let mut pos = self.character_index;

        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        let original = self.character_index;
        self.filter_input = chars[..pos]
            .iter()
            .chain(chars[original..].iter())
            .collect();

        self.character_index = pos;
        self.apply_filter(area);
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.filter_input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn apply_filter(&mut self, area: Rect) {
        let lower_filter = if !self.filter_input.is_empty() {
            self.filter_input.to_lowercase()
        } else {
            String::new()
        };

        let old_selected = self.themes.selected;

        if lower_filter.is_empty() {
            self.themes.filtered_indices = (0..self.themes.items.len()).collect();
        } else {
            self.themes.filtered_indices = self
                .themes
                .items
                .iter()
                .enumerate()
                .filter(|(_, theme)| theme.name.to_lowercase().contains(&lower_filter))
                .map(|(i, _)| i)
                .collect();
        }

        if !self.themes.filtered_indices.contains(&old_selected) {
            self.themes.selected = *self.themes.filtered_indices.first().unwrap_or(&0);
            self.update_cached_colors();
        }

        self.adjust_scroll(area);
    }

    fn filtered_pos(&self) -> usize {
        self.themes
            .filtered_indices
            .iter()
            .position(|&i| i == self.themes.selected)
            .unwrap_or(0)
    }

    fn visible_count(&self, area: Rect) -> usize {
        area.height.saturating_sub(3) as usize // footer (1) + top/bottom borders (2)
    }

    fn adjust_scroll(&mut self, area: Rect) {
        let vis = self.visible_count(area);
        let len = self.themes.filtered_indices.len();
        let max = len.saturating_sub(vis);
        let pos = self.filtered_pos();

        if pos < self.themes.scroll {
            self.themes.scroll = pos;
        } else if pos >= self.themes.scroll + vis {
            self.themes.scroll = (pos + 1).saturating_sub(vis).min(max);
        }
        self.themes.scroll = self.themes.scroll.min(max);
    }

    fn select_next(&mut self, area: Rect) {
        let pos = self.filtered_pos();
        if pos + 1 < self.themes.filtered_indices.len() {
            self.themes.selected = self.themes.filtered_indices[pos + 1];
            self.update_cached_colors();
            self.adjust_scroll(area);
        }
    }

    fn select_previous(&mut self) {
        let pos = self.filtered_pos();
        if pos > 0 {
            self.themes.selected = self.themes.filtered_indices[pos - 1];
            self.update_cached_colors();
            if pos - 1 < self.themes.scroll {
                self.themes.scroll = pos - 1;
            }
        }
    }

    fn page_down(&mut self, area: Rect) {
        let vis = self.visible_count(area) / 2;
        let pos = self.filtered_pos();
        let new_pos = (pos + vis).min(self.themes.filtered_indices.len() - 1);
        self.themes.selected = self.themes.filtered_indices[new_pos];
        self.update_cached_colors();
        let max_scroll = self
            .themes
            .filtered_indices
            .len()
            .saturating_sub(self.visible_count(area));
        self.themes.scroll = (new_pos + 1)
            .saturating_sub(self.visible_count(area) / 2)
            .min(max_scroll);
    }

    fn page_up(&mut self, area: Rect) {
        let vis = self.visible_count(area) / 2;
        let pos = self.filtered_pos();
        let new_pos = pos.saturating_sub(vis);
        self.themes.selected = self.themes.filtered_indices[new_pos];
        self.update_cached_colors();
        self.themes.scroll = new_pos.min(
            self.themes
                .filtered_indices
                .len()
                .saturating_sub(self.visible_count(area)),
        );
    }

    fn select_first(&mut self) {
        if let Some(&idx) = self.themes.filtered_indices.first() {
            self.themes.selected = idx;
            self.themes.scroll = 0;
            self.update_cached_colors();
        }
    }

    fn select_last(&mut self, area: Rect) {
        if let Some(&idx) = self.themes.filtered_indices.last() {
            self.themes.selected = idx;
            self.update_cached_colors();
            self.adjust_scroll(area);
        }
    }

    fn confirm_selection(&mut self) {
        let Some(theme) = self.themes.items.get(self.themes.selected) else {
            return;
        };

        match switch_theme(&theme.name, self.custom_themes_path.as_deref()) {
            Ok(applied) => {
                self.status_message = Some(format!("Applied theme: {}", applied.name));
            }
            Err(err) => {
                self.status_message = Some(format!("Failed to apply theme: {err}"));
            }
        }

        if self.quit_on_select {
            self.should_exit = true
        }
    }

    fn update_cached_colors(&mut self) {
        let Some(theme) = self.themes.items.get(self.themes.selected) else {
            self.themes.cached_colors = None;
            return;
        };

        self.themes.cached_colors = match ThemeColors::from_path(&theme.path) {
            Ok(c) => Some(c),
            Err(e) => {
                self.status_message = Some(format!("Failed to load preview: {e}"));
                None
            }
        };
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]);
        let [content_area, footer_area] = area.layout(&main_layout);

        let content_layout = Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)]);
        let [list_area, item_area] = content_area.layout(&content_layout);

        self.render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        if self.show_preview {
            self.render_preview(item_area, buf);
        }
    }
}

impl App {
    fn render_footer(&mut self, area: Rect, buf: &mut Buffer) {
        match self.input_mode {
            InputMode::Normal => {
                let mut left_spans: Vec<Span> = vec![];
                if let Some(msg) = &self.status_message {
                    left_spans.push(
                        Span::from(format!(" {} ", msg))
                            .fg(Color::Red)
                            .bg(SURFACE_COLOR),
                    );
                } else if let Some(theme) = self.themes.items.get(self.themes.selected) {
                    left_spans.push(
                        Span::from(format!(" {} ", theme.path.display()))
                            .fg(TEXT_COLOR)
                            .bg(SURFACE_COLOR),
                    );
                }

                let right_span =
                    Span::from(" enter: apply • /: filter • q: quit ").fg(SUBTEXT_COLOR);
                let right_line = Line::from(right_span);

                let footer_layout = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Length(right_line.width() as u16),
                ]);
                let [left_area, right_area] = area.layout(&footer_layout);

                Paragraph::new(Line::from(left_spans)).render(left_area, buf);
                Paragraph::new(right_line).render(right_area, buf);
            }
            InputMode::Filtering => {
                let spans = vec![
                    Span::from("/").italic().fg(Color::Yellow),
                    Span::from(&self.filter_input).fg(TEXT_COLOR),
                ];

                Paragraph::new(Line::from(spans)).render(area, buf);
            }
        }
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().padding(Padding::uniform(1));

        let vis = block.inner(area).height as usize;
        let start = self.themes.scroll;
        let end = (start + vis).min(self.themes.filtered_indices.len());

        let visible_items: Vec<ListItem> = self.themes.filtered_indices[start..end]
            .iter()
            .map(|&idx| ListItem::new(self.themes.items[idx].name.as_str()))
            .collect();

        let list = List::new(visible_items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("❯ ")
            .highlight_spacing(HighlightSpacing::Always);

        let mut temp_state = ListState::default();
        let relative = self.filtered_pos().saturating_sub(self.themes.scroll);
        temp_state.select(Some(relative));

        StatefulWidget::render(list, area, buf, &mut temp_state);
    }

    fn render_preview(&mut self, area: Rect, buf: &mut Buffer) {
        self.update_cached_colors();

        let colors = match &self.themes.cached_colors {
            Some(c) => c,
            None => {
                self.status_message = Some("Failed to load preview".to_string());
                return;
            }
        };

        let bg = colors.background();
        let fg = colors.foreground();
        let blue = colors.blue();
        let green = colors.green();
        let cyan = colors.cyan();
        let yellow = colors.yellow();
        let magenta = colors.magenta();

        let block = Block::default().padding(Padding::new(
            area.height / 4,
            area.width / 8,
            area.height / 8,
            area.width / 6,
        ));

        let prompt_line = Line::from(vec![
                Span::from("󰣇 ").fg(fg),
                Span::from("~/althemer ").fg(blue),
                Span::from(" main ").fg(green),
                Span::from(" v1.92.0 ").fg(cyan),
        ]);
        let text = vec![
            prompt_line.clone(),
            Line::from(vec![
                Span::from("❯ ").fg(magenta),
                Span::from("echo ").fg(fg),
                Span::from("'Alacritty is awesome!'").fg(yellow),
                Span::from("█").fg(colors.cursor_text()),
            ]),
            Line::from("\n"),
            prompt_line.clone(),
            Line::from(vec![Span::from("❯ ").fg(magenta), Span::from("ls ").fg(fg)]),
            Line::from(vec![
                Span::from(" Cargo.lock  ").fg(fg),
                Span::from(" Cargo.toml  ").fg(yellow),
                Span::from("󰂺 README.md  ").fg(yellow),
                Span::from("󰣞 src  ").fg(blue),
                Span::from(" target").fg(blue),
            ]),
        ];

        let mut inner_area = block.inner(area);
        let inner_block = Block::new().bg(bg).padding(Padding::proportional(1));
        inner_area.height = inner_area.height.max(16).min(area.height.saturating_sub(4));

        block.render(area, buf);
        Paragraph::new(text)
            .block(inner_block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .render(inner_area, buf);
    }
}
