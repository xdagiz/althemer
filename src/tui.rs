use crate::switcher::switch_theme;
use crate::themes::{Theme, ThemeColors, list_themes};
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::Padding;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget, Wrap,
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
}

struct ThemesList {
    items: Vec<Theme>,
    selected: usize,
    scroll: usize,
}

impl Default for App {
    fn default() -> Self {
        Self::new(None)
    }
}

impl App {
    pub fn new(custom_themes_path: Option<&Path>) -> Self {
        let (items, status_message) = match list_themes(custom_themes_path) {
            Ok(items) if items.is_empty() => (items, Some("No themes found".to_string())),
            Ok(items) => (items, None),
            Err(err) => (Vec::new(), Some(format!("Failed to load themes: {err}"))),
        };

        Self {
            should_exit: false,
            themes: ThemesList {
                items,
                selected: 0,
                scroll: 0,
            },
            status_message,
            custom_themes_path: custom_themes_path.map(Path::to_path_buf),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_exit {
            let area = Rect::from(terminal.size()?);
            terminal.draw(|frame| frame.render_widget(&mut self, area))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                self.handle_key(key, area);
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent, area: Rect) {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_exit = true
            }
            KeyCode::Char('q') => self.should_exit = true,
            KeyCode::Char('j') | KeyCode::Down => self.select_next(area),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
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
            KeyCode::Enter => self.confirm_selection(),
            _ => {}
        }
    }

    fn visible_height(&self, area: Rect) -> usize {
        area.height.saturating_sub(2) as usize
    }

    fn max_scroll(&self, visible_height: usize) -> usize {
        self.themes.items.len().saturating_sub(visible_height)
    }

    fn select_next(&mut self, area: Rect) {
        let vis = self.visible_height(area);
        let max = self.max_scroll(vis);

        if self.themes.selected < self.themes.items.len() - 1 {
            self.themes.selected += 1;

            if self.themes.selected >= self.themes.scroll + vis {
                self.themes.scroll = (self.themes.selected - vis + 1).min(max);
            }
        }
    }

    fn select_previous(&mut self) {
        if self.themes.selected > 0 {
            self.themes.selected -= 1;

            if self.themes.selected < self.themes.scroll {
                self.themes.scroll = self.themes.selected;
            }
        }
    }

    fn page_down(&mut self, area: Rect) {
        let vis = self.visible_height(area);
        let max = self.max_scroll(vis);

        self.themes.selected = (self.themes.selected + vis).min(self.themes.items.len() - 1);
        self.themes.scroll = (self.themes.scroll + vis).min(max);
    }

    fn page_up(&mut self, area: Rect) {
        let vis = self.visible_height(area);

        self.themes.selected = self.themes.selected.saturating_sub(vis);
        self.themes.scroll = self.themes.scroll.saturating_sub(vis);
    }

    fn select_first(&mut self) {
        self.themes.selected = 0;
        self.themes.scroll = 0;
    }

    fn select_last(&mut self, area: Rect) {
        let vis = self.visible_height(area);
        let max = self.max_scroll(vis);

        self.themes.selected = self.themes.items.len().saturating_sub(1);
        self.themes.scroll = max;
    }

    /// Applies the currently selected theme.
    fn confirm_selection(&mut self) {
        let index = self.themes.selected;

        let Some(theme) = self.themes.items.get(index) else {
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

        self.should_exit = false;
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
        self.render_selected_item(item_area, buf);
    }
}

/// Rendering logic for the app
impl App {
    fn render_footer(&mut self, area: Rect, buf: &mut Buffer) {
        let mut left_spans: Vec<Span> = vec![];
        if !self.themes.items.is_empty()
            && let Some(theme) = self.themes.items.get(self.themes.selected)
        {
            left_spans.extend(vec![Span::styled(
                format!(" {} ", theme.path.display()),
                Style::default().fg(TEXT_COLOR).bg(SURFACE_COLOR),
            )]);
        }

        let right_span = Span::styled("Enter: Apply", Style::default().fg(SUBTEXT_COLOR));
        let right_line = Line::from(right_span);

        let footer_layout = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(right_line.width() as u16),
        ]);
        let [left_area, right_area] = area.layout(&footer_layout);

        Paragraph::new(Line::from(left_spans)).render(left_area, buf);
        Paragraph::new(right_line).render(right_area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::raw(" Themes ").left_aligned())
            .border_type(BorderType::Rounded);

        let vis = self.visible_height(area);

        // Slice items to visible range using scroll offset
        let visible_items: Vec<ListItem> = self
            .themes
            .items
            .iter()
            .skip(self.themes.scroll)
            .take(vis)
            .map(|theme| ListItem::new(theme.name.as_str()))
            .collect();

        let list = List::new(visible_items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("❯ ")
            .highlight_spacing(HighlightSpacing::Always);

        // Create temporary ListState for highlighting the selected item
        let mut temp_state = ListState::default();
        let relative_selected = self.themes.selected.saturating_sub(self.themes.scroll);
        temp_state.select(Some(relative_selected));

        StatefulWidget::render(list, area, buf, &mut temp_state);
    }

    fn render_selected_item(&mut self, area: Rect, buf: &mut Buffer) {
        if !self.themes.items.is_empty()
            && let Some(theme) = self.themes.items.get(self.themes.selected)
        {
            let colors = match ThemeColors::from_path(&theme.path) {
                Ok(c) => c,
                Err(err) => {
                    self.status_message = Some(format!("Failed to load preview: {err}"));
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

            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::new(
                    area.height / 4, // top
                    area.width / 8,  // right
                    area.height / 8, // bottom
                    area.width / 4,  // left
                ))
                .title(Line::raw(" Preview ").left_aligned());

            let text = vec![
                Line::from(vec![
                    Span::from("󰣇 ").fg(fg),
                    Span::from("althemer ").fg(blue),
                    Span::from(" main ").fg(green),
                    Span::from(" v1.92.0 ").fg(cyan),
                ]),
                Line::from(vec![
                    Span::from("❯ ").fg(magenta),
                    Span::from("echo ").fg(fg),
                    Span::from("'Alacritty is awesome!'").fg(yellow),
                    Span::from("█").fg(colors.cursor_text()),
                ]),
            ];

            let mut inner_area = block.inner(area);
            inner_area.height = 20;

            let inner_block = Block::new().bg(bg).padding(Padding::proportional(1));

            block.render(area, buf);
            Paragraph::new(text)
                .block(inner_block)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true })
                .render(inner_area, buf);
        }
    }
}
