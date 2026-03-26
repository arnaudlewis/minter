use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use super::render::{AppState, PanelFocus};
use super::state::Action;

/// Result of handling an input event.
pub enum InputResult {
    /// No action taken
    None,
    /// Quit the application
    Quit,
    /// Trigger an action
    Action(Action),
    /// Selection or expansion changed (needs re-render)
    StateChanged,
}

/// Handle a crossterm event and produce an InputResult.
pub fn handle_event(event: &Event, app_state: &mut AppState, spec_count: usize) -> InputResult {
    match event {
        Event::Key(key) => handle_key_event(key, app_state, spec_count),
        Event::Mouse(mouse) => handle_mouse_event(mouse, app_state, spec_count),
        _ => InputResult::None,
    }
}

fn handle_key_event(key: &KeyEvent, app_state: &mut AppState, spec_count: usize) -> InputResult {
    // Ctrl+C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return InputResult::Quit;
    }

    match key.code {
        KeyCode::Char('q') => InputResult::Quit,
        KeyCode::Char('v') => InputResult::Action(Action::Validate),
        KeyCode::Char('d') => InputResult::Action(Action::DeepValidate),
        KeyCode::Char('c') => InputResult::Action(Action::Coverage),
        KeyCode::Char('n') => InputResult::Action(Action::Inspect),
        KeyCode::Char('l') => InputResult::Action(Action::Lock),
        KeyCode::Char('g') => InputResult::Action(Action::Graph),
        KeyCode::Tab => {
            app_state.toggle_focus();
            InputResult::StateChanged
        }
        KeyCode::Up => {
            if app_state.focus == PanelFocus::Integrity {
                app_state.integrity_scroll_up();
            } else {
                app_state.move_up();
            }
            InputResult::StateChanged
        }
        KeyCode::Down => {
            if app_state.focus == PanelFocus::Integrity {
                app_state.integrity_scroll_down();
            } else {
                app_state.move_down(spec_count);
            }
            InputResult::StateChanged
        }
        KeyCode::PageDown => {
            app_state.integrity_page_down(100);
            InputResult::StateChanged
        }
        KeyCode::PageUp => {
            app_state.integrity_page_up();
            InputResult::StateChanged
        }
        KeyCode::Enter => {
            app_state.toggle_expand();
            InputResult::StateChanged
        }
        KeyCode::Esc => {
            // Clear both action result and selection at once
            let had_result = app_state.action_result.is_some();
            let had_selection = app_state.selected_spec.is_some();
            app_state.action_result = None;
            app_state.selected_spec = None;
            if had_result || had_selection {
                InputResult::StateChanged
            } else {
                InputResult::None
            }
        }
        _ => InputResult::None,
    }
}

fn handle_mouse_event(
    mouse: &MouseEvent,
    app_state: &mut AppState,
    spec_count: usize,
) -> InputResult {
    match mouse.kind {
        MouseEventKind::Down(_) => {
            // Calculate which spec was clicked based on row
            // The specs panel starts at row 3 (after the overview bar border)
            // Each spec is one row, expanded behaviors add more rows
            // For simplicity, map mouse row to spec index
            let row = mouse.row as usize;
            if row >= 4 && spec_count > 0 {
                let clicked_idx = row.saturating_sub(4);
                if clicked_idx < spec_count {
                    if app_state.selected_spec == Some(clicked_idx) {
                        app_state.toggle_expand();
                    } else {
                        app_state.selected_spec = Some(clicked_idx);
                    }
                    return InputResult::StateChanged;
                }
            }
            InputResult::None
        }
        MouseEventKind::ScrollUp => {
            app_state.move_up();
            InputResult::StateChanged
        }
        MouseEventKind::ScrollDown => {
            app_state.move_down(spec_count);
            InputResult::StateChanged
        }
        _ => InputResult::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton,
        MouseEvent, MouseEventKind,
    };

    fn make_key_event(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        })
    }

    #[test]
    fn esc_clears_action_result_and_selection_at_once() {
        let mut state = AppState::new();
        state.selected_spec = Some(2);
        state.action_result = Some("validate output".to_string());

        let result = handle_event(&make_key_event(KeyCode::Esc), &mut state, 5);
        assert!(matches!(result, InputResult::StateChanged));
        assert!(state.action_result.is_none());
        assert_eq!(state.selected_spec, None);
    }

    #[test]
    fn esc_deselects_spec_when_no_action_result() {
        let mut state = AppState::new();
        state.selected_spec = Some(1);

        let result = handle_event(&make_key_event(KeyCode::Esc), &mut state, 5);
        assert!(matches!(result, InputResult::StateChanged));
        assert_eq!(state.selected_spec, None);
    }

    #[test]
    fn esc_with_no_selection_and_no_result_is_noop() {
        let mut state = AppState::new();

        let result = handle_event(&make_key_event(KeyCode::Esc), &mut state, 5);
        assert!(matches!(result, InputResult::None));
    }

    #[test]
    fn mouse_click_selects_spec() {
        let mut state = AppState::new();
        let click = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 5,
            modifiers: KeyModifiers::empty(),
        });
        let result = handle_event(&click, &mut state, 3);
        assert!(matches!(result, InputResult::StateChanged));
        assert_eq!(state.selected_spec, Some(1));
    }

    #[test]
    fn mouse_click_on_selected_spec_toggles_expand() {
        let mut state = AppState::new();
        state.selected_spec = Some(1);

        let click = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 5,
            modifiers: KeyModifiers::empty(),
        });
        let result = handle_event(&click, &mut state, 3);
        assert!(matches!(result, InputResult::StateChanged));
        assert!(state.expanded_specs.contains(&1));
    }
}
