use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::io;
use peroxide::{App, AppError, FormState, InputMode, FileBrowserMode, ConfirmationMode};

fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    
    if let Ok(connections) = App::load_connections() {
        app.connections = connections;
    }
    
    run(&mut terminal, app)?;
    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> Result<()> {
    if let Ok(additional_keys) = App::load_additional_keys() {
        for key in additional_keys {
            app.add_key_path(key);
        }
    }

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            app.clear_error();
            
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        app.save_connections()?;
                        return Ok(());
                    }
                    KeyCode::Char('a') => {
                        app.input_mode = InputMode::Adding;
                        app.form_state = FormState::new();
                    }
                    KeyCode::Char('e') => {
                        app.edit_connection();
                    }
                    KeyCode::Char('d') => {
                        app.delete_connection();
                    }
                    KeyCode::Char('y') => {
                        if let Err(e) = app.duplicate_connection() {
                            app.show_error(e);
                        }
                    }
                    KeyCode::Up => {
                        if let Some(selected) = app.selected_connection {
                            if selected > 0 {
                                app.selected_connection = Some(selected - 1);
                            }
                        } else if !app.connections.is_empty() {
                            app.selected_connection = Some(0);
                        }
                    }
                    KeyCode::Down => {
                        if let Some(selected) = app.selected_connection {
                            if selected < app.connections.len().saturating_sub(1) {
                                app.selected_connection = Some(selected + 1);
                            }
                        } else if !app.connections.is_empty() {
                            app.selected_connection = Some(0);
                        }
                    }
                    KeyCode::Char('c') => {
                        if let Some(idx) = app.selected_connection {
                            match app.test_connection(idx) {
                                Ok(_) => {
                                    match app.execute_ssh() {
                                        Ok(needs_redraw) => {
                                            if needs_redraw {
                                                terminal.clear()?;
                                                terminal.draw(|f| ui(f, &app))?;
                                            }
                                        }
                                        Err(e) => {
                                            app.show_error(format!("Failed to execute SSH: {}", e));
                                        }
                                    }
                                }
                                Err(e) => match e {
                                    AppError::ConnectionFailed(msg) => {
                                        app.show_error(format!("Connection test failed: {}", msg));
                                    }
                                    AppError::AuthenticationFailed(msg) => {
                                        app.show_error(format!("Authentication test failed: {}", msg));
                                    }
                                    AppError::NoConnectionSelected => {
                                        app.show_error("No connection selected");
                                    }
                                },
                            }
                        } else {
                            app.show_error("No connection selected");
                        }
                    }
                    KeyCode::Char('k') => {
                        if let Err(e) = app.select_key_file() {
                            app.show_error(e.to_string());
                        } else {
                            if let Err(e) = app.save_additional_keys() {
                                app.show_error(format!("Failed to save additional keys: {}", e));
                            }
                        }
                    }
                    KeyCode::Char('f') => {
                        if let Err(e) = app.select_key_folder() {
                            app.show_error(e.to_string());
                        } else {
                            if let Err(e) = app.save_additional_keys() {
                                app.show_error(format!("Failed to save additional keys: {}", e));
                            }
                        }
                    }
                    KeyCode::Char('t') => {
                        if let Some(idx) = app.selected_connection {
                            match app.test_connection(idx) {
                                Ok(_) => app.show_error("Connection test successful!"),
                                Err(e) => match e {
                                    AppError::ConnectionFailed(msg) => {
                                        app.show_error(format!("Connection test failed: {}", msg));
                                    }
                                    AppError::AuthenticationFailed(msg) => {
                                        app.show_error(format!("Authentication test failed: {}", msg));
                                    }
                                    AppError::NoConnectionSelected => {
                                        app.show_error("No connection selected");
                                    }
                                },
                            }
                        } else {
                            app.show_error("No connection selected");
                        }
                    }
                    KeyCode::Char('s') => {
                        app.input_mode = InputMode::Settings;
                    }
                    KeyCode::Enter => {
                        if let Some(idx) = app.selected_connection {
                            match app.test_connection(idx) {
                                Ok(_) => {
                                    match app.execute_ssh() {
                                        Ok(needs_redraw) => {
                                            if needs_redraw {
                                                terminal.clear()?;
                                                terminal.draw(|f| ui(f, &app))?;
                                            }
                                        }
                                        Err(e) => {
                                            app.show_error(format!("Failed to execute SSH: {}", e));
                                        }
                                    }
                                }
                                Err(e) => match e {
                                    AppError::ConnectionFailed(msg) => {
                                        app.show_error(format!("Connection test failed: {}", msg));
                                    }
                                    AppError::AuthenticationFailed(msg) => {
                                        app.show_error(format!("Authentication test failed: {}", msg));
                                    }
                                    AppError::NoConnectionSelected => {
                                        app.show_error("No connection selected");
                                    }
                                },
                            }
                        } else {
                            app.show_error("No connection selected");
                        }
                    }
                    _ => {}
                },
                InputMode::Adding | InputMode::Editing => match key.code {
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    KeyCode::Tab => app.next_field(),
                    KeyCode::BackTab => app.previous_field(),
                    KeyCode::Backspace => app.delete_char(),
                    KeyCode::Enter => {
                        let result = match app.input_mode {
                            InputMode::Adding => app.save_connection(),
                            InputMode::Editing => app.update_connection(),
                            _ => unreachable!(),
                        };
                        if let Err(e) = result {
                            app.show_error(e);
                        }
                    }
                    KeyCode::Char(c) => app.add_char(c),
                    KeyCode::Right => {
                        if app.form_state.active_field == 5 {
                            app.select_ssh_key(1)
                        }
                    },
                    KeyCode::Left => {
                        if app.form_state.active_field == 5 {
                            app.select_ssh_key(-1)
                        }
                    },
                    _ => {}
                },
                InputMode::Settings => match key.code {
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    KeyCode::Tab => app.next_settings_tab(),
                    KeyCode::Up => {
                        if app.settings_selected_item > 0 {
                            app.settings_selected_item -= 1;
                        }
                    }
                    KeyCode::Down => {
                        app.settings_selected_item += 1;
                    }
                    KeyCode::Char('d') => {
                        if app.settings_selected_item >= 3 && app.settings_selected_item < app.ssh_keys.len() + 3 {
                            let key_index = app.settings_selected_item - 3;
                            app.remove_ssh_key(key_index);
                            if let Err(e) = app.save_additional_keys() {
                                app.show_error(format!("Failed to save additional keys: {}", e));
                            }
                        }
                    }
                    KeyCode::Enter => {
                        match app.settings_selected_item {
                            0 => if let Err(e) = app.select_key_file() {
                                app.show_error(e.to_string());
                            },
                            1 => if let Err(e) = app.select_key_folder() {
                                app.show_error(e.to_string());
                            },
                            _ => {}
                        }
                        if let Err(e) = app.save_additional_keys() {
                            app.show_error(format!("Failed to save additional keys: {}", e));
                        }
                    }
                    _ => {}
                },
                InputMode::FileBrowser(mode) => match key.code {
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Settings;
                        app.file_browser = None;
                    }
                    KeyCode::Up => {
                        if let Some(browser) = &mut app.file_browser {
                            browser.move_up();
                        }
                    }
                    KeyCode::Down => {
                        if let Some(browser) = &mut app.file_browser {
                            browser.move_down();
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(browser) = &mut app.file_browser {
                            match mode {
                                FileBrowserMode::SingleFile => {
                                    if let Some(path) = browser.get_selected_path() {
                                        if path.is_dir() {
                                            browser.enter_directory();
                                        } else {
                                            if browser.is_valid_ssh_key(&path) {
                                                app.add_key_path(path);
                                                if let Err(e) = app.save_additional_keys() {
                                                    app.show_error(format!("Failed to save additional keys: {}", e));
                                                }
                                                app.input_mode = InputMode::Settings;
                                                app.file_browser = None;
                                            } else {
                                                app.show_error("Not a valid SSH key file");
                                            }
                                        }
                                    }
                                }
                                FileBrowserMode::Directory => {
                                    if let Some(path) = browser.get_selected_path() {
                                        if path == browser.current_path {
                                            let mut valid_paths = Vec::new();
                                            if let Ok(entries) = std::fs::read_dir(&path) {
                                                for entry in entries.flatten() {
                                                    let path = entry.path();
                                                    if browser.is_valid_ssh_key(&path) {
                                                        valid_paths.push(path);
                                                    }
                                                }
                                            }
                                            
                                            let added = valid_paths.len();
                                            for path in valid_paths {
                                                app.add_key_path(path);
                                            }
                                            
                                            if let Err(e) = app.save_additional_keys() {
                                                app.show_error(format!("Failed to save additional keys: {}", e));
                                            }
                                            app.show_error(format!("Added {} SSH keys from folder", added));
                                            app.input_mode = InputMode::Settings;
                                            app.file_browser = None;
                                        } else if path.ends_with("..") {
                                            browser.enter_directory();
                                        } else if path.is_dir() {
                                            browser.enter_directory();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
                InputMode::Confirmation(_mode) => match key.code {
                    KeyCode::Esc => app.cancel_confirmation(),
                    KeyCode::Left | KeyCode::Right => app.toggle_confirmation_selection(),
                    KeyCode::Enter => {
                        if app.confirmation_selected {
                            if let Err(e) = app.perform_confirmed_action() {
                                app.show_error(e);
                            } else {
                                app.save_connections()?;
                            }
                        }
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.area());

    let title = Paragraph::new("Peroxide - SSH Connection Manager")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    match &app.input_mode {
        InputMode::Normal => render_connections(f, app, chunks[1]),
        InputMode::Adding | InputMode::Editing => render_form(f, app, chunks[1]),
        InputMode::Settings => render_settings(f, app, chunks[1]),
        InputMode::FileBrowser(_mode) => render_file_browser(f, app, chunks[1]),
        InputMode::Confirmation(mode) => render_confirmation(f, app, chunks[1], mode),
    }

    let help = match &app.input_mode {
        InputMode::Normal => "q: Quit | a: Add | e: Edit | d: Delete | y: Duplicate | s: Settings | ↑↓: Navigate",
        InputMode::Adding => "Esc: Cancel | Tab: Next Field | Enter: Save | ←→: Select SSH Key",
        InputMode::Editing => "Esc: Cancel | Tab: Next Field | Enter: Update | ←→: Select SSH Key",
        InputMode::Settings => "Esc: Back | Tab: Switch Tab | ↑↓: Navigate | Enter: Select | d: Delete Key",
        InputMode::FileBrowser(_mode) => "Esc: Cancel | ↑↓: Navigate | Enter: Select/Enter Directory",
        InputMode::Confirmation(_) => "Esc: Cancel | ←→: Navigate | Enter: Confirm Selection",
    };

    let help = Paragraph::new(help)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);

    if let Some(error) = &app.error_message {
        let error_message = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_message, chunks[3]);
    }
}

fn render_connections(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .connections
        .iter()
        .map(|conn| {
            let auth_method = if conn.key_path.is_some() {
                "🔑"
            } else if conn.password.is_some() {
                "🔒"
            } else {
                "❌"
            };

            let status = match conn.last_connection_status {
                Some(true) => "✅",
                Some(false) => "❌",
                None => "  ",
            };
            
            ListItem::new(format!(
                "{} {} {} ({}@{}:{})",
                status, auth_method, conn.name, conn.username, conn.host, conn.port
            ))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Connections").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_stateful_widget(
        list,
        area,
        &mut ListState::default().with_selected(app.selected_connection),
    );
}

fn render_form(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    let form_fields = [
        ("Name", &app.form_state.name),
        ("Host", &app.form_state.host),
        ("Port", &app.form_state.port),
        ("Username", &app.form_state.username),
        ("Password", &app.form_state.password),
        ("Key Passphrase", &app.form_state.key_passphrase),
    ];

    for (i, (title, content)) in form_fields.iter().enumerate() {
        let style = if app.form_state.active_field == i {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let display_content = if (i == 4 || i == 5) && !content.is_empty() {
            "*".repeat(content.len())
        } else {
            content.to_string()
        };

        let input = Paragraph::new(display_content)
            .style(style)
            .block(Block::default().title(*title).borders(Borders::ALL));
        f.render_widget(input, chunks[i]);
    }

    let key_items = {
        let mut items = Vec::new();
        
        let is_none_selected = match app.form_state.selected_key {
            Some(0) => true,
            _ => false
        };
        
        let none_display_text = if is_none_selected {
            "《 none 》".to_string()
        } else {
            "  none  ".to_string()
        };
        
        items.push(Span::styled(
            none_display_text,
            if is_none_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }
        ));
        
        for (i, path) in app.ssh_keys.iter().enumerate() {
            let is_selected = app.form_state.selected_key == Some(i + 1);
            let file_name = path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let display_text = if is_selected {
                format!("《 {} 》", file_name)
            } else {
                format!("  {}  ", file_name)
            };

            items.push(Span::styled(
                display_text,
                if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                }
            ));
        }
        
        items
    };

    let key_text = Line::from(key_items);
    
    let key_paragraph = Paragraph::new(key_text)
        .alignment(Alignment::Center)
        .block(Block::default()
            .title("SSH Key (←→ to select)")
            .borders(Borders::ALL)
            .style(if app.form_state.active_field == 5 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }));

    f.render_widget(key_paragraph, chunks[6]);
}

fn render_settings(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ])
        .split(area);

    let tabs = vec!["SSH Keys"];
    let tabs = Tabs::new(tabs)
        .select(0)
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .highlight_style(Style::default().fg(Color::Yellow));
    f.render_widget(tabs, chunks[0]);

    let items = vec![
        ListItem::new("Add SSH Key File"),
        ListItem::new("Add SSH Key Folder"),
        ListItem::new("Current SSH Keys:"),
    ];

    let mut key_items: Vec<ListItem> = if let InputMode::Editing = app.input_mode {
        if app.form_state.selected_key == Some(0) {
            vec![ListItem::new("  none (current)")]
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    key_items.extend(app.ssh_keys
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let is_current = if let InputMode::Editing = app.input_mode {
                app.form_state.selected_key == Some(i + 1)
            } else {
                false
            };
            
            let label = if is_current {
                format!("  {} (current)", 
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                )
            } else {
                format!("  {}", 
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                )
            };
            
            ListItem::new(label)
        }));

    let mut all_items = items;
    all_items.append(&mut key_items);

    let list = List::new(all_items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(
        list,
        chunks[1],
        &mut ListState::default().with_selected(Some(app.settings_selected_item)),
    );
}

fn render_file_browser(f: &mut Frame, app: &App, area: Rect) {
    if let Some(browser) = &app.file_browser {
        let items: Vec<ListItem> = browser
            .entries
            .iter()
            .map(|path| {
                let name = browser.get_display_name(path);
                let prefix = if path.is_dir() { "📁 " } else { "📄 " };
                ListItem::new(format!("{}{}", prefix, name))
            })
            .collect();

        let title = format!("Browse: {}", browser.current_path.display());
        let list = List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        f.render_stateful_widget(
            list,
            area,
            &mut ListState::default().with_selected(Some(browser.selected)),
        );
    }
}

fn render_confirmation(f: &mut Frame, app: &App, area: Rect, mode: &ConfirmationMode) {
    let prompt = match mode {
        ConfirmationMode::Delete => "Are you sure you want to delete this connection?",
        ConfirmationMode::Duplicate => "Are you sure you want to duplicate this connection?",
        ConfirmationMode::Update => "Are you sure you want to save these changes?",
    };

    let dialog_area = Rect {
        x: area.x + area.width / 4,
        y: area.y + area.height / 3,
        width: area.width / 2,
        height: area.height / 3,
    };

    let dialog = Block::default()
        .title(prompt)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(Clear, dialog_area);
    f.render_widget(dialog, dialog_area);

    let centered_button_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(dialog_area);
    
    let left_half = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Length(12),
            Constraint::Percentage(40),
        ])
        .split(centered_button_layout[0]);
    
    let right_half = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(12),
            Constraint::Percentage(60),
        ])
        .split(centered_button_layout[1]);
    
    let button_y = dialog_area.y + (dialog_area.height * 2/3) - 2;
    let button_height = 3;
    
    let no_button_area = Rect {
        x: left_half[1].x,
        y: button_y,
        width: left_half[1].width,
        height: button_height,
    };
    
    let yes_button_area = Rect {
        x: right_half[1].x,
        y: button_y,
        width: right_half[1].width,
        height: button_height,
    };

    let no_style = if !app.confirmation_selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
    } else {
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray)
    };

    let yes_style = if app.confirmation_selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
    } else {
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray)
    };

    let no_button = Paragraph::new(" No ")
        .alignment(Alignment::Center)
        .style(no_style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(
                if !app.confirmation_selected {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default()
                }
            ));
    
    let yes_button = Paragraph::new(" Yes ")
        .alignment(Alignment::Center)
        .style(yes_style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(
                if app.confirmation_selected {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default()
                }
            ));
    
    f.render_widget(no_button, no_button_area);
    f.render_widget(yes_button, yes_button_area);
} 