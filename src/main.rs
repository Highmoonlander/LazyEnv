mod app;
mod ui;
mod python;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::{App, AppState, DialogState, Focus};
use crate::ui::ui;
use crate::python::{list_environments, list_packages, create_environment, delete_environment, install_package, uninstall_package};

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    
    // Load initial data
    match list_environments() {
        Ok(envs) => {
            app.environments = envs;
            if !app.environments.is_empty() {
                app.selected_environment = Some(0);
                // Don't load packages initially to avoid errors
            }
        },
        Err(e) => {
            eprintln!("Error loading environments: {}", e);
            // Continue with empty environments list
        }
    }

    // Main loop
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::Normal => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down => {
                            if app.focus == Focus::Environments {
                                app.next_environment();
                            } else if app.focus == Focus::Packages {
                                app.next_package();
                            }
                        },
                        KeyCode::Up => {
                            if app.focus == Focus::Environments {
                                app.previous_environment();
                            } else if app.focus == Focus::Packages {
                                app.previous_package();
                            }
                        },
                        KeyCode::Tab => app.toggle_focus(),
                        KeyCode::Enter => {
                            if let Some(idx) = app.selected_environment {
                                match list_packages(&app.environments[idx].path) {
                                    Ok(pkgs) => {
                                        app.packages = pkgs;
                                        if !app.packages.is_empty() {
                                            app.selected_package = Some(0);
                                        }
                                    },
                                    Err(e) => {
                                        app.status_message = Some(format!("Error listing packages: {}", e));
                                    }
                                }
                            }
                        },
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Char('n') => {
                            app.state = AppState::CreateEnvironment;
                            app.input_text.clear();
                        },
                        KeyCode::Char('d') => {
                            if let Some(idx) = app.selected_environment {
                                app.state = AppState::DeleteEnvironment;
                                app.dialog_state = DialogState::Confirm;
                            }
                        },
                        KeyCode::Char('i') => {
                            if let Some(_) = app.selected_environment {
                                app.state = AppState::InstallPackage;
                                app.input_text.clear();
                            }
                        },
                        KeyCode::Char('r') => {
                            if let Some(_) = app.selected_environment {
                                if let Some(pkg_idx) = app.selected_package {
                                    if pkg_idx < app.packages.len() {
                                        app.state = AppState::UninstallPackage;
                                        app.dialog_state = DialogState::Confirm;
                                    }
                                }
                            }
                        },
                        KeyCode::Char('s') => {
                            app.state = AppState::SearchEnvironment;
                            app.input_text.clear();
                        },
                        KeyCode::Char('g') => {
                            app.show_global_packages = !app.show_global_packages;
                            if app.show_global_packages {
                                match python::list_global_packages() {
                                    Ok(pkgs) => {
                                        app.packages = pkgs;
                                        if !app.packages.is_empty() {
                                            app.selected_package = Some(0);
                                        }
                                    },
                                    Err(e) => {
                                        app.status_message = Some(format!("Error listing global packages: {}", e));
                                    }
                                }
                            } else if let Some(idx) = app.selected_environment {
                                match list_packages(&app.environments[idx].path) {
                                    Ok(pkgs) => {
                                        app.packages = pkgs;
                                        if !app.packages.is_empty() {
                                            app.selected_package = Some(0);
                                        }
                                    },
                                    Err(e) => {
                                        app.status_message = Some(format!("Error listing packages: {}", e));
                                    }
                                }
                            }
                        },
                        KeyCode::Char('R') => {
                            // Refresh environments
                            match list_environments() {
                                Ok(envs) => {
                                    app.environments = envs;
                                    if !app.environments.is_empty() {
                                        app.selected_environment = Some(0);
                                        app.status_message = Some("Environments refreshed".to_string());
                                    }
                                },
                                Err(e) => {
                                    app.status_message = Some(format!("Error refreshing environments: {}", e));
                                }
                            }
                        },
                        KeyCode::Char('x') => {
                            app.state = AppState::HelpMenu;
                        },
                        _ => {}
                    },
                    AppState::PackageView => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down => app.next_package(),
                        KeyCode::Up => app.previous_package(),
                        KeyCode::Tab => app.toggle_focus(),
                        KeyCode::Esc => app.state = AppState::Normal,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Char('i') => {
                            if let Some(_) = app.selected_environment {
                                app.state = AppState::InstallPackage;
                                app.input_text.clear();
                            }
                        },
                        KeyCode::Char('r') => {
                            if let Some(_) = app.selected_environment {
                                if let Some(pkg_idx) = app.selected_package {
                                    if pkg_idx < app.packages.len() {
                                        app.state = AppState::UninstallPackage;
                                        app.dialog_state = DialogState::Confirm;
                                    }
                                }
                            }
                        },
                        KeyCode::Char('x') => {
                            app.state = AppState::HelpMenu;
                        },
                        _ => {}
                    },
                    AppState::HelpMenu => match key.code {
                        KeyCode::Esc | KeyCode::Char('x') => {
                            app.state = AppState::Normal;
                        },
                        _ => {}
                    },
                    AppState::CreateEnvironment => match key.code {
                        KeyCode::Esc => {
                            app.state = AppState::Normal;
                        },
                        KeyCode::Enter => {
                            if !app.input_text.is_empty() {
                                match create_environment(&app.input_text) {
                                    Ok(env) => {
                                        app.environments.push(env);
                                        app.selected_environment = Some(app.environments.len() - 1);
                                        match list_packages(&app.environments[app.environments.len() - 1].path) {
                                            Ok(pkgs) => {
                                                app.packages = pkgs;
                                                if !app.packages.is_empty() {
                                                    app.selected_package = Some(0);
                                                }
                                            },
                                            Err(e) => {
                                                app.status_message = Some(format!("Error listing packages: {}", e));
                                            }
                                        }
                                        app.state = AppState::Normal;
                                        app.status_message = Some(format!("Environment '{}' created successfully", app.input_text));
                                    },
                                    Err(e) => {
                                        app.status_message = Some(format!("Error creating environment: {}", e));
                                    }
                                }
                            }
                        },
                        KeyCode::Char(c) => {
                            app.input_text.push(c);
                        },
                        KeyCode::Backspace => {
                            app.input_text.pop();
                        },
                        _ => {}
                    },
                    AppState::DeleteEnvironment => match key.code {
                        KeyCode::Esc => {
                            app.state = AppState::Normal;
                            app.dialog_state = DialogState::None;
                        },
                        KeyCode::Char('y') => {
                            if let Some(idx) = app.selected_environment {
                                let env_path = app.environments[idx].path.clone();
                                let env_name = app.environments[idx].name.clone();
                                match delete_environment(&env_path) {
                                    Ok(_) => {
                                        app.environments.remove(idx);
                                        if app.environments.is_empty() {
                                            app.selected_environment = None;
                                            app.packages.clear();
                                        } else {
                                            app.selected_environment = Some(idx.min(app.environments.len() - 1));
                                            match list_packages(&app.environments[app.selected_environment.unwrap()].path) {
                                                Ok(pkgs) => {
                                                    app.packages = pkgs;
                                                    if !app.packages.is_empty() {
                                                        app.selected_package = Some(0);
                                                    }
                                                },
                                                Err(e) => {
                                                    app.status_message = Some(format!("Error listing packages: {}", e));
                                                }
                                            }
                                        }
                                        app.status_message = Some(format!("Environment '{}' deleted successfully", env_name));
                                    },
                                    Err(e) => {
                                        app.status_message = Some(format!("Error deleting environment: {}", e));
                                    }
                                }
                            }
                            app.state = AppState::Normal;
                            app.dialog_state = DialogState::None;
                        },
                        KeyCode::Char('n') => {
                            app.state = AppState::Normal;
                            app.dialog_state = DialogState::None;
                        },
                        _ => {}
                    },
                    AppState::InstallPackage => match key.code {
                        KeyCode::Esc => {
                            app.state = AppState::Normal;
                        },
                        KeyCode::Enter => {
                            if !app.input_text.is_empty() && app.selected_environment.is_some() {
                                let idx = app.selected_environment.unwrap();
                                let env_path = &app.environments[idx].path;
                                match install_package(env_path, &app.input_text) {
                                    Ok(_) => {
                                        match list_packages(env_path) {
                                            Ok(pkgs) => {
                                                app.packages = pkgs;
                                                if !app.packages.is_empty() {
                                                    app.selected_package = Some(0);
                                                }
                                            },
                                            Err(e) => {
                                                app.status_message = Some(format!("Error listing packages: {}", e));
                                            }
                                        }
                                        app.status_message = Some(format!("Package '{}' installed successfully", app.input_text));
                                    },
                                    Err(e) => {
                                        app.status_message = Some(format!("Error installing package: {}", e));
                                    }
                                }
                            }
                            app.state = AppState::Normal;
                        },
                        KeyCode::Char(c) => {
                            app.input_text.push(c);
                        },
                        KeyCode::Backspace => {
                            app.input_text.pop();
                        },
                        _ => {}
                    },
                    AppState::UninstallPackage => match key.code {
                        KeyCode::Esc => {
                            app.state = AppState::Normal;
                            app.dialog_state = DialogState::None;
                        },
                        KeyCode::Char('y') => {
                            if let Some(env_idx) = app.selected_environment {
                                if let Some(pkg_idx) = app.selected_package {
                                    if pkg_idx < app.packages.len() {
                                        let env_path = &app.environments[env_idx].path;
                                        let pkg_name = app.packages[pkg_idx].name.clone();
                                        match uninstall_package(env_path, &pkg_name) {
                                            Ok(_) => {
                                                match list_packages(env_path) {
                                                    Ok(pkgs) => {
                                                        app.packages = pkgs;
                                                        app.selected_package = Some(pkg_idx.min(app.packages.len().saturating_sub(1)));
                                                    },
                                                    Err(e) => {
                                                        app.status_message = Some(format!("Error listing packages: {}", e));
                                                    }
                                                }
                                                app.status_message = Some(format!("Package '{}' uninstalled successfully", pkg_name));
                                            },
                                            Err(e) => {
                                                app.status_message = Some(format!("Error uninstalling package: {}", e));
                                            }
                                        }
                                    }
                                }
                            }
                            app.state = AppState::Normal;
                            app.dialog_state = DialogState::None;
                        },
                        KeyCode::Char('n') => {
                            app.state = AppState::Normal;
                            app.dialog_state = DialogState::None;
                        },
                        _ => {}
                    },
                    AppState::SearchEnvironment => match key.code {
                        KeyCode::Esc => {
                            app.state = AppState::Normal;
                        },
                        KeyCode::Enter => {
                            if !app.input_text.is_empty() {
                                let search_term = app.input_text.to_lowercase();
                                let filtered_envs = app.environments.iter().enumerate()
                                    .filter(|(_, env)| env.name.to_lowercase().contains(&search_term) || 
                                                      env.path.to_string_lossy().to_lowercase().contains(&search_term))
                                    .map(|(idx, _)| idx)
                                    .collect::<Vec<_>>();
                                
                                if !filtered_envs.is_empty() {
                                    app.selected_environment = Some(filtered_envs[0]);
                                    match list_packages(&app.environments[filtered_envs[0]].path) {
                                        Ok(pkgs) => {
                                            app.packages = pkgs;
                                            if !app.packages.is_empty() {
                                                app.selected_package = Some(0);
                                            }
                                        },
                                        Err(e) => {
                                            app.status_message = Some(format!("Error listing packages: {}", e));
                                        }
                                    }
                                    app.status_message = Some(format!("Found {} matching environments", filtered_envs.len()));
                                } else {
                                    app.status_message = Some("No matching environments found".to_string());
                                }
                            }
                            app.state = AppState::Normal;
                        },
                        KeyCode::Char(c) => {
                            app.input_text.push(c);
                        },
                        KeyCode::Backspace => {
                            app.input_text.pop();
                        },
                        _ => {}
                    },
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
            
            // Clear status message after a delay
            if let Some(_) = &app.status_message {
                app.status_message_timer += 1;
                if app.status_message_timer > 20 { // ~2 seconds with 100ms tick rate
                    app.status_message = None;
                    app.status_message_timer = 0;
                }
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

    Ok(())
}

