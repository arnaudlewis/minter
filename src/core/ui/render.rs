use std::collections::HashSet;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use super::state::{IntegrityStatus, UiState, ValidationStatus};

// ── Panel focus ──────────────────────────────────────────

/// Which panel currently owns keyboard navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    Specs,
    Integrity,
}

// ── AppState ─────────────────────────────────────────────

/// Application state for the TUI (selection, scroll, expanded specs, etc.)
pub struct AppState {
    pub selected_spec: Option<usize>,
    pub expanded_specs: HashSet<usize>,
    pub scroll_offset: usize,
    pub integrity_scroll: usize,
    pub focus: PanelFocus,
    pub action_result: Option<String>,
    pub quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Assumed visible rows in the specs viewport for scroll tracking.
const VIEWPORT_HEIGHT: usize = 20;

impl AppState {
    pub fn new() -> Self {
        Self {
            selected_spec: None,
            expanded_specs: HashSet::new(),
            scroll_offset: 0,
            integrity_scroll: 0,
            focus: PanelFocus::Specs,
            action_result: None,
            quit: false,
        }
    }

    pub fn toggle_expand(&mut self) {
        if let Some(idx) = self.selected_spec {
            if self.expanded_specs.contains(&idx) {
                self.expanded_specs.remove(&idx);
            } else {
                self.expanded_specs.insert(idx);
            }
        }
    }

    pub fn move_up(&mut self) {
        match self.selected_spec {
            Some(0) => {
                self.selected_spec = None;
                self.scroll_offset = 0;
            }
            Some(n) => {
                let new = n - 1;
                self.selected_spec = Some(new);
                if new < self.scroll_offset {
                    self.scroll_offset = new;
                }
            }
            None => {
                // No selection — noop
            }
        }
    }

    pub fn move_down(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        match self.selected_spec {
            None => {
                self.selected_spec = Some(0);
            }
            Some(n) if n < max - 1 => {
                let new = n + 1;
                self.selected_spec = Some(new);
                if new >= self.scroll_offset + VIEWPORT_HEIGHT {
                    self.scroll_offset = new + 1 - VIEWPORT_HEIGHT;
                }
            }
            _ => {}
        }
    }

    pub fn integrity_page_down(&mut self, content_lines: usize) {
        let new = self.integrity_scroll + 10;
        if content_lines > 0 && new < content_lines {
            self.integrity_scroll = new;
        } else if content_lines > 0 {
            self.integrity_scroll = content_lines.saturating_sub(1);
        } else {
            self.integrity_scroll = new;
        }
    }

    pub fn integrity_page_up(&mut self) {
        self.integrity_scroll = self.integrity_scroll.saturating_sub(10);
    }

    pub fn integrity_scroll_down(&mut self) {
        self.integrity_scroll += 1;
    }

    pub fn integrity_scroll_up(&mut self) {
        self.integrity_scroll = self.integrity_scroll.saturating_sub(1);
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            PanelFocus::Specs => PanelFocus::Integrity,
            PanelFocus::Integrity => PanelFocus::Specs,
        };
    }
}

// ── Render ───────────────────────────────────────────────

/// Render the full UI frame.
pub fn render(f: &mut Frame, ui_state: &UiState, app_state: &AppState) {
    let size = f.area();

    // Main layout: top bar, middle content, bottom bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // overview bar
            Constraint::Min(10),   // main content
            Constraint::Length(3), // action bar
        ])
        .split(size);

    render_overview_bar(f, ui_state, main_chunks[0]);

    // Middle: left (specs list) + right (integrity / results)
    let mid_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    render_specs_panel(f, ui_state, app_state, mid_chunks[0]);

    if let Some(ref result) = app_state.action_result {
        render_result_panel(f, result, mid_chunks[1]);
    } else {
        render_integrity_panel(f, ui_state, app_state, mid_chunks[1]);
    }

    let has_selection = app_state.selected_spec.is_some();
    render_action_bar(f, main_chunks[2], has_selection, &app_state.focus);
}

fn render_overview_bar(f: &mut Frame, state: &UiState, area: Rect) {
    let lock_label = if state.lock_aligned() {
        Span::styled(" aligned ", Style::default().fg(Color::Green))
    } else if state.integrity().lock_status == IntegrityStatus::NoLock {
        Span::styled(" no lock ", Style::default().fg(Color::Yellow))
    } else {
        Span::styled(" drifted ", Style::default().fg(Color::Red))
    };

    let coverage_color = if state.coverage_percent() >= 80 {
        Color::Green
    } else if state.coverage_percent() >= 50 {
        Color::Yellow
    } else {
        Color::Red
    };

    let line = Line::from(vec![
        Span::styled(" Specs: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("{}", state.spec_count())),
        Span::raw("  "),
        Span::styled("Behaviors: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("{}", state.behavior_count())),
        Span::raw("  "),
        Span::styled("NFRs: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("{}", state.nfr_count())),
        Span::raw("  "),
        Span::styled("Tags: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("{}", state.test_count())),
        Span::raw("  "),
        Span::styled("Coverage: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{}%", state.coverage_percent()),
            Style::default().fg(coverage_color),
        ),
        Span::raw("  "),
        Span::styled("Lock: ", Style::default().add_modifier(Modifier::BOLD)),
        lock_label,
    ]);

    let block = Block::default().borders(Borders::ALL).title(" minter ");
    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}

fn render_specs_panel(f: &mut Frame, ui_state: &UiState, app_state: &AppState, area: Rect) {
    let specs = ui_state.specs_list();
    let mut items: Vec<ListItem> = Vec::new();

    for (idx, spec) in specs.iter().enumerate() {
        let selected = app_state.selected_spec == Some(idx);
        let expanded = app_state.expanded_specs.contains(&idx);
        let marker = if expanded { "v" } else { ">" };

        // Validation status icon
        let (status_icon, status_color) = match &spec.validation_status {
            ValidationStatus::Valid => ("\u{2713}", Color::Green), // ✓
            ValidationStatus::Invalid(_) => ("\u{2717}", Color::Red), // ✗
            ValidationStatus::Unknown => ("?", Color::DarkGray),
        };

        let line = if selected {
            let style = Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD);
            Line::from(vec![
                Span::styled(format!(" {} ", status_icon), style),
                Span::styled(format!(" {} ", marker), style),
                Span::styled(format!("{} v{}", spec.name, spec.version), style),
                Span::styled(format!("  ({} behaviors)", spec.behavior_count), style),
            ])
        } else {
            Line::from(vec![
                Span::styled(
                    format!(" {} ", status_icon),
                    Style::default().fg(status_color),
                ),
                Span::raw(format!(" {} ", marker)),
                Span::raw(format!("{} v{}", spec.name, spec.version)),
                Span::styled(
                    format!("  ({} behaviors)", spec.behavior_count),
                    Style::default().add_modifier(Modifier::DIM),
                ),
            ])
        };
        items.push(ListItem::new(line));

        if expanded {
            for behavior in &spec.behaviors {
                let (icon, icon_color) = if behavior.covered {
                    ("\u{2713}", Color::Green)
                } else {
                    ("\u{2717}", Color::Red)
                };

                let badge = if behavior.test_types.is_empty() {
                    Span::styled("uncovered", Style::default().fg(Color::Red))
                } else {
                    let types_str = format!("[{}]", behavior.test_types.join(", "));
                    Span::styled(types_str, Style::default().fg(Color::Cyan))
                };

                let bline = Line::from(vec![
                    Span::styled(format!("    {} ", icon), Style::default().fg(icon_color)),
                    Span::raw(format!("{} ", behavior.name)),
                    badge,
                ]);
                items.push(ListItem::new(bline));
            }
        }
    }

    if ui_state.has_error() {
        if let Some(msg) = ui_state.error_message() {
            let line = Line::from(vec![Span::styled(
                format!(" Error: {}", msg),
                Style::default().fg(Color::Red),
            )]);
            items.push(ListItem::new(line));
        }
    }

    let block = Block::default().borders(Borders::ALL).title(" Specs ");

    // Apply scroll_offset — skip items before the viewport
    let visible_items: Vec<ListItem> = items.into_iter().skip(app_state.scroll_offset).collect();

    let list = List::new(visible_items).block(block);
    f.render_widget(list, area);
}

fn render_integrity_panel(f: &mut Frame, ui_state: &UiState, app_state: &AppState, area: Rect) {
    let integrity = ui_state.integrity();
    let drift = ui_state.drift_details();

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(""));

    // ── Drift status ────────────────────────────────────

    // Specs status
    let (specs_icon, specs_color) = status_display(&integrity.specs);
    lines.push(Line::from(vec![
        Span::styled(
            format!(" {} ", specs_icon),
            Style::default().fg(specs_color),
        ),
        Span::styled("Specs: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            status_label(&integrity.specs),
            Style::default().fg(specs_color),
        ),
    ]));
    for s in &drift.modified_specs {
        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled(
                format!("{} (modified)", s),
                Style::default().fg(Color::Yellow),
            ),
        ]));
    }
    for s in &drift.unlocked_specs {
        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled(
                format!("{} (unlocked)", s),
                Style::default().fg(Color::Yellow),
            ),
        ]));
    }

    // NFRs status
    let (nfrs_icon, nfrs_color) = status_display(&integrity.nfrs);
    lines.push(Line::from(vec![
        Span::styled(format!(" {} ", nfrs_icon), Style::default().fg(nfrs_color)),
        Span::styled("NFRs: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            status_label(&integrity.nfrs),
            Style::default().fg(nfrs_color),
        ),
    ]));
    for s in &drift.unlocked_nfrs {
        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled(
                format!("{} (unlocked)", s),
                Style::default().fg(Color::Yellow),
            ),
        ]));
    }

    // Tests status
    let (tests_icon, tests_color) = status_display(&integrity.tests);
    lines.push(Line::from(vec![
        Span::styled(
            format!(" {} ", tests_icon),
            Style::default().fg(tests_color),
        ),
        Span::styled("Tests: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            status_label(&integrity.tests),
            Style::default().fg(tests_color),
        ),
    ]));
    for s in &drift.missing_tests {
        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled(format!("{} (missing)", s), Style::default().fg(Color::Red)),
        ]));
    }

    // ── Validation errors ───────────────────────────────

    let invalid_specs: Vec<_> = ui_state
        .specs_list()
        .iter()
        .filter_map(|s| {
            if let ValidationStatus::Invalid(ref errs) = s.validation_status {
                Some((&s.name, errs))
            } else {
                None
            }
        })
        .collect();

    if !invalid_specs.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!(" \u{2717} Validation errors ({})", invalid_specs.len()),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]));
        for (name, errs) in &invalid_specs {
            lines.push(Line::from(vec![Span::styled(
                format!("   \u{2717} {}", name),
                Style::default().fg(Color::Red),
            )]));
            let max_show = 3;
            for (i, err) in errs.iter().enumerate() {
                if i >= max_show {
                    let remaining = errs.len() - max_show;
                    lines.push(Line::from(vec![Span::styled(
                        format!("       +{} more", remaining),
                        Style::default().add_modifier(Modifier::DIM),
                    )]));
                    break;
                }
                lines.push(Line::from(vec![Span::styled(
                    format!("       {}", err),
                    Style::default().add_modifier(Modifier::DIM),
                )]));
            }
        }
    }

    // ── Uncovered behaviors ─────────────────────────────

    let mut uncovered_by_spec: Vec<(&str, Vec<&str>)> = Vec::new();
    for spec in ui_state.specs_list() {
        let uncovered: Vec<&str> = spec
            .behaviors
            .iter()
            .filter(|b| !b.covered)
            .map(|b| b.name.as_str())
            .collect();
        if !uncovered.is_empty() {
            uncovered_by_spec.push((&spec.name, uncovered));
        }
    }

    if !uncovered_by_spec.is_empty() {
        let total_uncovered: usize = uncovered_by_spec.iter().map(|(_, bs)| bs.len()).sum();
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!(" Uncovered behaviors ({})", total_uncovered),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        for (spec_name, behaviors) in &uncovered_by_spec {
            lines.push(Line::from(vec![Span::styled(
                format!("   {}", spec_name),
                Style::default().add_modifier(Modifier::BOLD),
            )]));
            for behavior in behaviors {
                lines.push(Line::from(vec![Span::styled(
                    format!("     {}", behavior),
                    Style::default().fg(Color::Yellow),
                )]));
            }
        }
    }

    // ── Invalid tags ────────────────────────────────────

    let invalid_tags = ui_state.invalid_tags();
    if !invalid_tags.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!(" Invalid tags ({})", invalid_tags.len()),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]));
        for tag in invalid_tags {
            lines.push(Line::from(vec![Span::styled(
                format!("   {}:{} \u{2014} {}", tag.file, tag.line, tag.message),
                Style::default().add_modifier(Modifier::DIM),
            )]));
        }
    }

    // ── Dependency errors ────────────────────────────

    let dep_errors = ui_state.dep_errors();
    if !dep_errors.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!(" Dependency errors ({})", dep_errors.len()),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]));
        for err in dep_errors {
            lines.push(Line::from(vec![Span::styled(
                format!("   {}", err),
                Style::default().add_modifier(Modifier::DIM),
            )]));
        }
    }

    // Clamp scroll so we can't scroll past content
    let content_height = lines.len();
    let visible_height = area.height.saturating_sub(2) as usize; // minus borders
    let max_scroll = content_height.saturating_sub(visible_height);
    let clamped_scroll = app_state.integrity_scroll.min(max_scroll);

    let block = Block::default().borders(Borders::ALL).title(" Integrity ");
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((clamped_scroll as u16, 0));
    f.render_widget(paragraph, area);
}

fn render_result_panel(f: &mut Frame, result: &str, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Results ");
    let paragraph = Paragraph::new(result.to_string())
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn render_action_bar(f: &mut Frame, area: Rect, has_selection: bool, focus: &PanelFocus) {
    let focus_label = match focus {
        PanelFocus::Specs => "specs",
        PanelFocus::Integrity => "integrity",
    };

    let mut spans = vec![
        Span::styled(" [v]", Style::default().fg(Color::Cyan)),
        Span::raw(" validate  "),
        Span::styled("[c]", Style::default().fg(Color::Cyan)),
        Span::raw(" coverage  "),
    ];

    if has_selection {
        spans.push(Span::styled("[d]", Style::default().fg(Color::Cyan)));
        spans.push(Span::raw(" deep  "));
    }

    spans.push(Span::styled("[g]", Style::default().fg(Color::Cyan)));
    spans.push(Span::raw(" graph  "));

    if has_selection {
        spans.push(Span::styled("[n]", Style::default().fg(Color::Cyan)));
        spans.push(Span::raw(" inspect  "));
    }

    spans.push(Span::styled("[l]", Style::default().fg(Color::Cyan)));
    spans.push(Span::raw(" lock  "));

    spans.push(Span::styled("[Tab]", Style::default().fg(Color::Cyan)));
    spans.push(Span::raw(format!(" {}  ", focus_label)));

    if has_selection {
        spans.push(Span::styled("[Esc]", Style::default().fg(Color::Yellow)));
        spans.push(Span::raw(" all  "));
    }

    spans.push(Span::styled("[q]", Style::default().fg(Color::Red)));
    spans.push(Span::raw(" quit"));

    let line = Line::from(spans);
    let block = Block::default().borders(Borders::ALL).title(" Actions ");
    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}

fn status_display(status: &IntegrityStatus) -> (&str, Color) {
    match status {
        IntegrityStatus::Aligned => ("\u{2713}", Color::Green),
        IntegrityStatus::Drifted => ("!", Color::Yellow),
        IntegrityStatus::NoLock => ("?", Color::DarkGray),
    }
}

fn status_label(status: &IntegrityStatus) -> &str {
    match status {
        IntegrityStatus::Aligned => "aligned",
        IntegrityStatus::Drifted => "drifted",
        IntegrityStatus::NoLock => "no lock",
    }
}
