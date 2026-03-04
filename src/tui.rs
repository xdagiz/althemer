use crate::switcher::switch_theme;
use crate::themes::{Theme, list_themes};
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{
    Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
    StatefulWidget, Widget, Wrap,
};
use ratatui::{DefaultTerminal, symbols};
use std::io;
use std::path::{Path, PathBuf};

const CATPPUCCIN_TEXT: Color = Color::Rgb(205, 214, 244);
const CATPPUCCIN_SUBTEXT0: Color = Color::Rgb(166, 173, 200);
const CATPPUCCIN_SURFACE0: Color = Color::Rgb(49, 50, 68);

const LIST_HEADER_STYLE: Style = Style::new().fg(CATPPUCCIN_SUBTEXT0);
const SELECTED_STYLE: Style = Style::new()
    .fg(CATPPUCCIN_TEXT)
    .bg(CATPPUCCIN_SURFACE0)
    .add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = CATPPUCCIN_TEXT;

pub struct App {
    should_exit: bool,
    themes: ThemesList,
    status_message: Option<String>,
    custom_themes_path: Option<PathBuf>,
}

struct ThemesList {
    items: Vec<Theme>,
    state: ListState,
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

        let mut state = ListState::default();
        if !items.is_empty() {
            state.select_first();
        }

        Self {
            should_exit: false,
            themes: ThemesList { items, state },
            status_message,
            custom_themes_path: custom_themes_path.map(Path::to_path_buf),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_exit = true
            }
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Enter => self.confirm_selection(),
            _ => {}
        }
    }

    fn select_next(&mut self) {
        self.themes.state.select_next();
    }

    fn select_previous(&mut self) {
        self.themes.state.select_previous();
    }

    fn select_first(&mut self) {
        self.themes.state.select_first();
    }

    fn select_last(&mut self) {
        self.themes.state.select_last();
    }

    /// Applies the currently selected theme.
    fn confirm_selection(&mut self) {
        let Some(index) = self.themes.state.selected() else {
            return;
        };

        let Some(theme) = self.themes.items.get(index) else {
            return;
        };

        let theme_name = theme.name.clone();

        match switch_theme(&theme_name, self.custom_themes_path.as_deref()) {
            Ok(applied) => {
                self.status_message = Some(format!("Applied theme: {}", applied.name));
            }
            Err(err) => {
                self.status_message = Some(format!("Failed to apply theme: {err}"));
            }
        }

        self.should_exit = true;
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [header_area, content_area, footer_area] = area.layout(&main_layout);

        let content_layout = Layout::horizontal([Constraint::Percentage(45), Constraint::Fill(1)]);
        let [list_area, item_area] = content_area.layout(&content_layout);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);
    }
}

/// Rendering logic for the app
impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Alacritty Theme Switcher")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new(
            "Use ↓↑ to move, Enter to apply theme, g/G to go top/bottom, Ctrl+c/q/Esc to quit.",
        )
        .centered()
        .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Themes").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(LIST_HEADER_STYLE);

        let items: Vec<ListItem> = self
            .themes
            .items
            .iter()
            .map(|theme| ListItem::new(theme.name.as_str()))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.themes.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        if let Some(index) = self.themes.state.selected() {
            if let Some(theme) = self.themes.items.get(index) {
                lines.push(format!("Selected: {}", theme.name));
                lines.push(format!("Path: {}", theme.path.display()));
            } else {
                lines.push("Nothing selected...".to_string());
            }
        } else {
            lines.push("Nothing selected...".to_string());
        }

        if let Some(status) = &self.status_message {
            lines.push(String::new());
            lines.push(status.clone());
        }

        let info = lines.join("\n");

        let block = Block::new()
            .title(Line::raw("Theme Details").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(LIST_HEADER_STYLE)
            .padding(Padding::horizontal(1));

        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}
