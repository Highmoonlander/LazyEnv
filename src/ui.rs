use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    Frame,
};

use crate::app::{App, AppState, DialogState, Focus};

pub fn ui(f: &mut Frame, app: &mut App) {
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.size());

    let main_area = chunks[0];
    let status_area = chunks[1];

    // Split main area into sidebar and content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(main_area);

    // Draw environments sidebar
    render_environments(f, app, main_chunks[0]);
    
    // Draw packages panel
    render_packages(f, app, main_chunks[1]);

    // Render status bar
    render_status_bar(f, app, status_area);

    // Render dialogs on top if needed
    match app.state {
        AppState::CreateEnvironment => {
            render_input_dialog(f, "Create New Environment", "Enter environment name:", &app.input_text);
        },
        AppState::DeleteEnvironment => {
            if app.dialog_state == DialogState::Confirm {
                if let Some(idx) = app.selected_environment {
                    let env_name = &app.environments[idx].name;
                    render_confirm_dialog(f, "Delete Environment", &format!("Are you sure you want to delete '{}'? (y/n)", env_name));
                }
            }
        },
        AppState::InstallPackage => {
            render_input_dialog(f, "Install Package", "Enter package name:", &app.input_text);
        },
        AppState::UninstallPackage => {
            if app.dialog_state == DialogState::Confirm {
                if let Some(pkg_idx) = app.selected_package {
                    if pkg_idx < app.packages.len() {
                        let pkg_name = &app.packages[pkg_idx].name;
                        render_confirm_dialog(f, "Uninstall Package", &format!("Are you sure you want to uninstall '{}'? (y/n)", pkg_name));
                    }
                }
            }
        },
        AppState::SearchEnvironment => {
            render_input_dialog(f, "Search Environments", "Enter search term:", &app.input_text);
        },
        AppState::HelpMenu => {
            render_help_menu(f);
        },
        _ => {}
    }
}

fn render_environments(f: &mut Frame, app: &App, area: Rect) {
    let title = if app.show_global_packages {
        "Python Environments (Global Packages)"
    } else {
        "Python Environments"
    };

    // Set border color based on focus
    let border_style = if app.focus == Focus::Environments {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let environments: Vec<ListItem> = app
        .environments
        .iter()
        .map(|env| {
            let env_type = match &env.env_type[..] {
                "venv" => "venv",
                "conda" => "conda",
                "pyenv" => "pyenv",
                "system" => "system",
                _ => "unknown",
            };
            
            ListItem::new(format!("{} ({}) [{}]", env.name, env.python_version, env_type))
        })
        .collect();

    let environments_list = List::new(environments)
        .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(app.selected_environment);

    f.render_stateful_widget(environments_list, area, &mut state);
}

fn render_packages(f: &mut Frame, app: &App, area: Rect) {
    // Split the right panel into two parts: packages list and details
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(area);

    // Determine title based on global package view
    let title = if app.show_global_packages {
        "Global Packages"
    } else {
        &if let Some(idx) = app.selected_environment {
            format!("Packages in {}", app.environments[idx].name)
        } else {
            "Packages".to_string()
        }
    };

    // Set border color based on focus
    let border_style = if app.focus == Focus::Packages {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    // Render packages list
    let packages: Vec<ListItem> = app
        .packages
        .iter()
        .map(|pkg| {
            ListItem::new(format!("{} ({})", pkg.name, pkg.version))
        })
        .collect();

    let packages_list = List::new(packages)
        .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(app.selected_package);

    f.render_stateful_widget(packages_list, chunks[0], &mut state);

    // Render package details
    let details = if let Some(idx) = app.selected_package {
        if idx < app.packages.len() {
            let pkg = &app.packages[idx];
            format!(
                "Name: {}
Version: {}
Summary: {}",
                pkg.name, pkg.version, pkg.summary
            )
        } else {
            "No package selected".to_string()
        }
    } else {
        "No package selected".to_string()
    };

    let details_widget = Paragraph::new(details)
        .block(Block::default().title("Package Details").borders(Borders::ALL));

    f.render_widget(details_widget, chunks[1]);

    // Render help text at the bottom
    let help_text = match app.state {
        AppState::Normal => {
            if app.focus == Focus::Environments {
                "Press 'x' for help | Tab: Switch focus | Enter: View packages"
            } else {
                "Press 'x' for help | Tab: Switch focus"
            }
        },
        AppState::PackageView => "Press 'x' for help | Tab: Switch focus | Esc: Back",
        _ => "",
    };

    let help_widget = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray));

    let help_area = Rect {
        x: area.x,
        y: area.height + area.y - 1,
        width: area.width,
        height: 1,
    };

    f.render_widget(help_widget, help_area);
}

fn render_help_menu(f: &mut Frame) {
    let area = centered_rect(70, 70, f.size());
    
    // Clear the area
    f.render_widget(Clear, area);
    
    // Create a block for the help menu
    let help_block = Block::default()
        .title("LazyEnv Help")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));
    
    f.render_widget(help_block, area);
    
    // Create the inner area for content
    let inner_area = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width - 4,
        height: area.height - 2,
    };
    
    // Help content
    let help_content = "
NAVIGATION
↑/↓: Navigate through list
Tab: Switch focus between environments and packages
Enter: View packages for selected environment

ENVIRONMENT MANAGEMENT
n: Create new environment
d: Delete selected environment
s: Search environments
g: Toggle between environment packages and global packages
R: Refresh environment list

PACKAGE MANAGEMENT
i: Install package in selected environment
r: Remove selected package

OTHER
x: Show/hide this help menu
q: Quit application
Esc: Go back / Cancel current operation
";
    
    let help_widget = Paragraph::new(help_content)
        .style(Style::default().fg(Color::White));
    
    f.render_widget(help_widget, inner_area);
    
    // Render footer
    let footer_area = Rect {
        x: inner_area.x,
        y: inner_area.y + inner_area.height - 1,
        width: inner_area.width,
        height: 1,
    };
    
    let footer_widget = Paragraph::new("Press 'x' or Esc to close this menu")
        .style(Style::default().fg(Color::Yellow));
    
    f.render_widget(footer_widget, footer_area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_text = match &app.status_message {
        Some(msg) => msg.clone(),
        None => {
            if let Some(idx) = app.selected_environment {
                format!("Environment: {} | Path: {}", 
                    app.environments[idx].name,
                    app.environments[idx].path.display())
            } else {
                "No environment selected".to_string()
            }
        }
    };

    let status_style = if app.status_message.is_some() {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let status_widget = Paragraph::new(status_text)
        .style(status_style);

    f.render_widget(status_widget, area);
}

fn render_input_dialog(f: &mut Frame, title: &str, prompt: &str, input: &str) {
    let area = centered_rect(60, 6, f.size());
    
    // Clear the area
    f.render_widget(Clear, area);
    
    // Create a block for the dialog
    let dialog = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));
    
    f.render_widget(dialog, area);
    
    // Create the inner area for content
    let inner_area = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width - 4,
        height: area.height - 2,
    };
    
    // Render prompt
    let prompt_area = Rect {
        x: inner_area.x,
        y: inner_area.y,
        width: inner_area.width,
        height: 1,
    };
    
    let prompt_widget = Paragraph::new(prompt);
    f.render_widget(prompt_widget, prompt_area);
    
    // Render input field
    let input_area = Rect {
        x: inner_area.x,
        y: inner_area.y + 1,
        width: inner_area.width,
        height: 1,
    };
    
    let input_text = format!("> {}", input);
    let input_widget = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White));
    
    f.render_widget(input_widget, input_area);
    
    // Render help text
    let help_area = Rect {
        x: inner_area.x,
        y: inner_area.y + 3,
        width: inner_area.width,
        height: 1,
    };
    
    let help_widget = Paragraph::new("Enter: Confirm | Esc: Cancel")
        .style(Style::default().fg(Color::Gray));
    
    f.render_widget(help_widget, help_area);
}

fn render_confirm_dialog(f: &mut Frame, title: &str, message: &str) {
    let area = centered_rect(60, 6, f.size());
    
    // Clear the area
    f.render_widget(Clear, area);
    
    // Create a block for the dialog
    let dialog = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));
    
    f.render_widget(dialog, area);
    
    // Create the inner area for content
    let inner_area = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width - 4,
        height: area.height - 2,
    };
    
    // Render message
    let message_area = Rect {
        x: inner_area.x,
        y: inner_area.y,
        width: inner_area.width,
        height: 2,
    };
    
    let message_widget = Paragraph::new(message);
    f.render_widget(message_widget, message_area);
    
    // Render help text
    let help_area = Rect {
        x: inner_area.x,
        y: inner_area.y + 3,
        width: inner_area.width,
        height: 1,
    };
    
    let help_widget = Paragraph::new("y: Yes | n: No | Esc: Cancel")
        .style(Style::default().fg(Color::Gray));
    
    f.render_widget(help_widget, help_area);
}

/// Helper function to create a centered rect using up certain percentage of the available rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

