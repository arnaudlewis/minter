use std::io;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use super::input::{self, InputResult};
use super::render::{self, AppState};
use super::state::{Action, ActionResult, UiState};

/// Run the interactive TUI. Returns exit code.
pub fn run_ui(working_dir: &Path) -> i32 {
    match run_ui_inner(working_dir) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: TUI failed: {}", e);
            1
        }
    }
}

fn run_ui_inner(working_dir: &Path) -> Result<i32, io::Error> {
    // Install panic hook to restore terminal before printing the panic message.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load initial state
    let mut ui_state = UiState::load(working_dir);
    ui_state.validate_all();
    let mut app_state = AppState::new();

    // Setup file watcher
    let (watcher_tx, watcher_rx) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(300), watcher_tx)
        .map_err(|e| io::Error::other(e.to_string()))?;

    // Watch specs directory
    let config = crate::core::config::load_config(working_dir).ok();
    if let Some(ref config) = config {
        if config.specs.exists() {
            let _ = debouncer
                .watcher()
                .watch(&config.specs, notify::RecursiveMode::Recursive);
        }
        for test_dir in &config.tests {
            if test_dir.exists() {
                let _ = debouncer
                    .watcher()
                    .watch(test_dir, notify::RecursiveMode::Recursive);
            }
        }
    }

    // Main event loop
    loop {
        // Render
        terminal.draw(|f| render::render(f, &ui_state, &app_state))?;

        if app_state.quit {
            break;
        }

        // Poll for events (keyboard/mouse) with a short timeout
        if event::poll(Duration::from_millis(50))? {
            let evt = event::read()?;
            let spec_count = ui_state.spec_count();
            match input::handle_event(&evt, &mut app_state, spec_count) {
                InputResult::Quit => {
                    app_state.quit = true;
                }
                InputResult::Action(action) => {
                    let is_lock = matches!(action, Action::Lock);
                    let selected_spec_path = app_state
                        .selected_spec
                        .and_then(|idx| ui_state.specs_list().get(idx).map(|s| s.path.clone()));
                    let result = ui_state.run_action(action, selected_spec_path.as_deref());
                    app_state.action_result = Some(format_action_result(&result));
                    // Refresh state after actions that modify project state
                    if is_lock {
                        ui_state.refresh(working_dir);
                        ui_state.validate_all();
                    }
                }
                InputResult::StateChanged => {
                    // Will re-render on next loop iteration
                }
                InputResult::None => {}
            }
        }

        // Check for file watcher events (non-blocking)
        if let Ok(Ok(events)) = watcher_rx.try_recv() {
            let has_relevant = events.iter().any(|e| {
                if e.kind != DebouncedEventKind::Any {
                    return false;
                }
                let ext = e.path.extension().and_then(|ext| ext.to_str());
                matches!(ext, Some("spec" | "nfr" | "rs"))
            });
            if has_relevant {
                ui_state.refresh(working_dir);
                ui_state.validate_all();
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(0)
}

fn format_action_result(result: &ActionResult) -> String {
    match result {
        ActionResult::Validate { output, has_errors } => {
            if *has_errors {
                format!("Validation errors:\n\n{}", output)
            } else {
                format!("All specs valid\n\n{}", output)
            }
        }
        ActionResult::Coverage {
            covered,
            total,
            percent,
            uncovered_behaviors,
        } => {
            let mut s = format!("Coverage: {}/{} ({}%)\n", covered, total, percent);
            if !uncovered_behaviors.is_empty() {
                s.push_str("\nUncovered behaviors:\n");
                for b in uncovered_behaviors {
                    s.push_str(&format!("  - {}\n", b));
                }
            }
            s
        }
        ActionResult::Lock { success, message } => {
            if *success {
                format!("Lock: {}", message)
            } else {
                format!("Lock failed: {}", message)
            }
        }
        ActionResult::DeepValidate { output, has_errors } => {
            if *has_errors {
                format!("Deep validation errors:\n\n{}", output)
            } else {
                format!("Deep validation passed\n\n{}", output)
            }
        }
        ActionResult::Graph { output } => {
            format!("Dependency Graph:\n\n{}", output)
        }
        ActionResult::Inspect {
            output,
            behavior_count: _,
            categories: _,
            dependencies: _,
        } => {
            format!("Inspect:\n\n{}", output)
        }
        ActionResult::Format { output } => {
            format!("DSL Grammar Reference:\n\n{}", output)
        }
        ActionResult::Scaffold { output } => {
            format!("Scaffold:\n\n{}", output)
        }
        ActionResult::Guide { topics } => {
            let mut s = String::from("Guide Topics:\n\n");
            for t in topics {
                s.push_str(&format!("  - {}\n", t));
            }
            s
        }
    }
}
