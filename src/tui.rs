use crate::switcher::switch_theme;
use crate::themes::{Theme, list_themes};
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
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
    show_help: bool,
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
            show_help: false,
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
            KeyCode::Char('?') => self.show_help = !self.show_help,
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
        let main_layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [header_area, content_area, footer_area] = area.layout(&main_layout);

        let content_layout = Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)]);
        let [list_area, item_area] = content_area.layout(&content_layout);

        if self.show_help {
            App::render_help(area, buf);
        }

        App::render_header(header_area, buf);
        self.render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);
    }
}

/// Rendering logic for the app
impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Althemer - Alacritty Theme Switcher")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(&mut self, area: Rect, buf: &mut Buffer) {
        let mut left_spans: Vec<Span> = vec![];
        match self.themes.state.selected() {
            Some(i) => match self.themes.items.get(i) {
                Some(theme) => {
                    left_spans.extend(vec![Span::styled(
                        format!(" {} ", theme.path.display()),
                        Style::default().fg(TEXT_COLOR).bg(SURFACE_COLOR),
                    )]);
                }
                None => todo!(),
            },
            None => todo!(),
        }

        let right_span = Span::styled(" ? help ", Style::default().fg(SUBTEXT_COLOR));
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
            .title(Line::raw("Themes").left_aligned())
            .border_type(BorderType::Rounded);

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
        match self.themes.state.selected() {
            Some(i) => match self.themes.items.get(i) {
                Some(theme) => {
                    let colors = &theme.colors;

                    let block = Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title(Line::raw("Preview").left_aligned())
                        .padding(Padding::new(1, 2, 1, 2))
                        .bg(colors.background());

                    let text = vec![
                        Line::from(vec![
                            Span::from("󰣇 ").fg(colors.foreground()),
                            Span::from("althemer ").fg(colors.blue()),
                            Span::from(" main ").fg(colors.green()),
                            Span::from(" v1.92.0 ").fg(colors.cyan()),
                        ]),
                        Line::from(vec![
                            Span::from("❯ ").fg(colors.magenta()),
                            Span::from("echo ").fg(colors.foreground()),
                            Span::from("'Alacritty is awesome!'").fg(colors.yellow()),
                            Span::from("█").fg(colors.cursor()),
                        ]),
                    ];

                    Paragraph::new(text)
                        .block(block)
                        .alignment(Alignment::Left)
                        .wrap(Wrap { trim: true })
                        .render(area, buf);
                }
                None => todo!(),
            },
            None => todo!(),
        }
    }

    fn render_help(area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("Keymaps");
        let area = popup_area(area, 30, 60);
        block.render(area, buf);
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
