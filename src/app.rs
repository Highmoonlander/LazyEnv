use crate::python::{PythonEnvironment, Package};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Normal,
    PackageView,
    CreateEnvironment,
    DeleteEnvironment,
    InstallPackage,
    UninstallPackage,
    SearchEnvironment,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DialogState {
    None,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Environments,
    Packages,
}

pub struct App {
    pub state: AppState,
    pub dialog_state: DialogState,
    pub environments: Vec<PythonEnvironment>,
    pub selected_environment: Option<usize>,
    pub packages: Vec<Package>,
    pub selected_package: Option<usize>,
    pub focus: Focus,
    pub input_text: String,
    pub status_message: Option<String>,
    pub status_message_timer: u8,
    pub show_global_packages: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Normal,
            dialog_state: DialogState::None,
            environments: Vec::new(),
            selected_environment: None,
            packages: Vec::new(),
            selected_package: None,
            focus: Focus::Environments,
            input_text: String::new(),
            status_message: None,
            status_message_timer: 0,
            show_global_packages: false,
        }
    }

    pub fn next_environment(&mut self) {
        if self.focus != Focus::Environments {
            return;
        }
        
        let len = self.environments.len();
        if len > 0 {
            self.selected_environment = match self.selected_environment {
                Some(i) => Some((i + 1) % len),
                None => Some(0),
            };
        }
    }

    pub fn previous_environment(&mut self) {
        if self.focus != Focus::Environments {
            return;
        }
        
        let len = self.environments.len();
        if len > 0 {
            self.selected_environment = match self.selected_environment {
                Some(i) => Some((i + len - 1) % len),
                None => Some(len - 1),
            };
        }
    }

    pub fn next_package(&mut self) {
        if self.focus != Focus::Packages {
            return;
        }
        
        let len = self.packages.len();
        if len > 0 {
            self.selected_package = match self.selected_package {
                Some(i) => Some((i + 1) % len),
                None => Some(0),
            };
        }
    }

    pub fn previous_package(&mut self) {
        if self.focus != Focus::Packages {
            return;
        }
        
        let len = self.packages.len();
        if len > 0 {
            self.selected_package = match self.selected_package {
                Some(i) => Some((i + len - 1) % len),
                None => Some(len - 1),
            };
        }
    }

    pub fn toggle_focus(&mut self) {
        match self.focus {
            Focus::Environments => {
                self.focus = Focus::Packages;
                if self.packages.is_empty() {
                    self.selected_package = None;
                } else if self.selected_package.is_none() {
                    self.selected_package = Some(0);
                }
            },
            Focus::Packages => {
                self.focus = Focus::Environments;
                if self.environments.is_empty() {
                    self.selected_environment = None;
                } else if self.selected_environment.is_none() {
                    self.selected_environment = Some(0);
                }
            },
        }
    }
}

