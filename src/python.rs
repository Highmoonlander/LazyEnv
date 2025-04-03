use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

#[derive(Debug, Clone)]
pub struct PythonEnvironment {
    pub name: String,
    pub path: PathBuf,
    pub python_version: String,
    pub env_type: String, // "venv", "conda", "pyenv", "system"
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub summary: String,
}

pub fn list_environments() -> io::Result<Vec<PythonEnvironment>> {
    let mut environments = Vec::new();
    
    // Check for system Python
    if let Err(e) = detect_system_python(&mut environments) {
        eprintln!("Warning: Failed to detect system Python: {}", e);
    }
    
    // Check for virtualenv environments in common locations
    if let Err(e) = detect_venv_environments(&mut environments) {
        eprintln!("Warning: Failed to detect venv environments: {}", e);
    }
    
    // Check for pyenv environments
    if let Err(e) = detect_pyenv_environments(&mut environments) {
        eprintln!("Warning: Failed to detect pyenv environments: {}", e);
    }
    
    // Check for conda environments
    if let Err(e) = detect_conda_environments(&mut environments) {
        eprintln!("Warning: Failed to detect conda environments: {}", e);
    }
    
    // Check for environments in the current directory
    if let Err(e) = detect_local_environments(&mut environments) {
        eprintln!("Warning: Failed to detect local environments: {}", e);
    }
    
    Ok(environments)
}

fn detect_system_python(environments: &mut Vec<PythonEnvironment>) -> io::Result<()> {
    // Try to get system Python
    let output = Command::new("python")
        .args(["--version"])
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let version = if version.is_empty() {
                String::from_utf8_lossy(&output.stderr).trim().to_string()
            } else {
                version
            };
            
            // Get executable path
            let output = Command::new("python")
                .args(["-c", "import sys; print(sys.executable)"])
                .output()?;
            
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                
                environments.push(PythonEnvironment {
                    name: "System Python".to_string(),
                    path: PathBuf::from(path),
                    python_version: version,
                    env_type: "system".to_string(),
                });
            }
        }
    }
    
    // Also try python3
    let output = Command::new("python3")
        .args(["--version"])
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let version = if version.is_empty() {
                String::from_utf8_lossy(&output.stderr).trim().to_string()
            } else {
                version
            };
            
            // Get executable path
            let output = Command::new("python3")
                .args(["-c", "import sys; print(sys.executable)"])
                .output()?;
            
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let path_buf = PathBuf::from(path);
                
                // Check if this is different from the previous python
                if environments.iter().all(|env| env.path != path_buf) {
                    environments.push(PythonEnvironment {
                        name: "System Python 3".to_string(),
                        path: path_buf,
                        python_version: version,
                        env_type: "system".to_string(),
                    });
                }
            }
        }
    }
    
    Ok(())
}

fn detect_venv_environments(environments: &mut Vec<PythonEnvironment>) -> io::Result<()> {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    
    // Check for environments in ~/.virtualenvs (common for virtualenvwrapper)
    let virtualenvs_dir = home_dir.join(".virtualenvs");
    if virtualenvs_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&virtualenvs_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() && is_virtualenv(&path) {
                    if let Some(env) = create_environment_from_path(&path, "venv") {
                        environments.push(env);
                    }
                }
            }
        }
    }
    
    // Check for environments in ~/.venv (another common location)
    let venv_dir = home_dir.join(".venv");
    if venv_dir.is_dir() && is_virtualenv(&venv_dir) {
        if let Some(env) = create_environment_from_path(&venv_dir, "venv") {
            environments.push(env);
        }
    }
    
    Ok(())
}

fn detect_pyenv_environments(environments: &mut Vec<PythonEnvironment>) -> io::Result<()> {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    
    // Check for pyenv versions
    let pyenv_versions_dir = home_dir.join(".pyenv").join("versions");
    if pyenv_versions_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&pyenv_versions_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    // Check if this is a Python installation
                    let bin_dir = path.join("bin");
                    let python_exec = bin_dir.join("python");
                    
                    if python_exec.exists() {
                        let name = path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        
                        // Get Python version
                        let output = Command::new(&python_exec)
                            .args(["--version"])
                            .output();
                        
                        if let Ok(output) = output {
                            if output.status.success() {
                                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                                let version = if version.is_empty() {
                                    String::from_utf8_lossy(&output.stderr).trim().to_string()
                                } else {
                                    version
                                };
                                
                                environments.push(PythonEnvironment {
                                    name: format!("pyenv: {}", name),
                                    path: path.clone(),
                                    python_version: version,
                                    env_type: "pyenv".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn detect_conda_environments(environments: &mut Vec<PythonEnvironment>) -> io::Result<()> {
    // Try to get conda environments using 'conda env list'
    let output = Command::new("conda")
        .args(["env", "list", "--json"])
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let json_output = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_output) {
                if let Some(envs) = json.get("envs").and_then(|e| e.as_array()) {
                    for env in envs {
                        if let Some(path_str) = env.as_str() {
                            let path = PathBuf::from(path_str);
                            
                            // Get the name from the path
                            let name = path.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            
                            // Check for Python executable
                            let python_exec = path.join("bin").join("python");
                            let python_exec = if python_exec.exists() {
                                python_exec
                            } else {
                                path.join("python.exe") // Windows
                            };
                            
                            if python_exec.exists() {
                                // Get Python version
                                let output = Command::new(&python_exec)
                                    .args(["--version"])
                                    .output();
                                
                                if let Ok(output) = output {
                                    if output.status.success() {
                                        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                                        let version = if version.is_empty() {
                                            String::from_utf8_lossy(&output.stderr).trim().to_string()
                                        } else {
                                            version
                                        };
                                        
                                        environments.push(PythonEnvironment {
                                            name: format!("conda: {}", name),
                                            path: path.clone(),
                                            python_version: version,
                                            env_type: "conda".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn detect_local_environments(environments: &mut Vec<PythonEnvironment>) -> io::Result<()> {
    // Check for venv directories in the current directory
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                if is_virtualenv(&path) {
                    if let Some(env) = create_environment_from_path(&path, "venv") {
                        environments.push(env);
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn is_virtualenv(path: &Path) -> bool {
    // Check for common virtualenv directory structure
    let bin_dir = if cfg!(windows) {
        path.join("Scripts")
    } else {
        path.join("bin")
    };
    
    let python_exec = if cfg!(windows) {
        bin_dir.join("python.exe")
    } else {
        bin_dir.join("python")
    };
    
    let activate_script = if cfg!(windows) {
        bin_dir.join("activate.bat")
    } else {
        bin_dir.join("activate")
    };
    
    python_exec.exists() && activate_script.exists()
}

fn create_environment_from_path(path: &Path, env_type: &str) -> Option<PythonEnvironment> {
    let name = path.file_name()?.to_string_lossy().to_string();
    
    // Get Python version
    let python_path = if cfg!(windows) {
        path.join("Scripts").join("python.exe")
    } else {
        path.join("bin").join("python")
    };
    
    let output = Command::new(&python_path)
        .args(["--version"])
        .output()
        .ok()?;
    
    let version = if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            String::from_utf8_lossy(&output.stderr).trim().to_string()
        } else {
            stdout
        }
    } else {
        "Unknown".to_string()
    };
    
    Some(PythonEnvironment {
        name,
        path: path.to_path_buf(),
        python_version: version,
        env_type: env_type.to_string(),
    })
}

pub fn list_packages(env_path: &Path) -> io::Result<Vec<Package>> {
    let mut packages = Vec::new();
    
    // Try to find pip in different locations
    let possible_pip_paths = vec![
        if cfg!(windows) {
            env_path.join("Scripts").join("pip.exe")
        } else {
            env_path.join("bin").join("pip")
        },
        if cfg!(windows) {
            env_path.join("Scripts").join("pip3.exe")
        } else {
            env_path.join("bin").join("pip3")
        },
        // For system Python, try to use the Python executable to run pip as a module
        if cfg!(windows) {
            env_path.join("python.exe")
        } else {
            env_path.join("bin").join("python")
        },
    ];
    
    for pip_path in possible_pip_paths {
        if !pip_path.exists() {
            continue;
        }
        
        // If this is a Python executable, use it to run pip as a module
        let output = if pip_path.file_name().map_or(false, |name| name == "python" || name == "python.exe") {
            Command::new(&pip_path)
                .args(["-m", "pip", "list", "--format=json"])
                .output()
        } else {
            Command::new(&pip_path)
                .args(["list", "--format=json"])
                .output()
        };
        
        match output {
            Ok(output) if output.status.success() => {
                let json_output = String::from_utf8_lossy(&output.stdout);
                match serde_json::from_str::<Vec<serde_json::Value>>(&json_output) {
                    Ok(pkg_list) => {
                        for pkg in pkg_list {
                            if let (Some(name), Some(version)) = (
                                pkg.get("name").and_then(|n| n.as_str()),
                                pkg.get("version").and_then(|v| v.as_str()),
                            ) {
                                packages.push(Package {
                                    name: name.to_string(),
                                    version: version.to_string(),
                                    summary: pkg.get("summary")
                                        .and_then(|s| s.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                });
                            }
                        }
                        return Ok(packages);
                    },
                    Err(e) => {
                        eprintln!("Warning: Failed to parse pip output: {}", e);
                        // Try the next pip path
                    }
                }
            },
            _ => {
                // Try the next pip path
            }
        }
    }
    
    // If we get here, we couldn't find pip or it failed to run
    // Try using the Python executable directly to get installed packages
    let python_path = if cfg!(windows) {
        env_path.join("Scripts").join("python.exe")
    } else {
        env_path.join("bin").join("python")
    };
    
    if python_path.exists() {
        let script = r#"
import sys
import json
import pkg_resources

packages = []
for pkg in pkg_resources.working_set:
    packages.append({
        "name": pkg.project_name,
        "version": pkg.version,
        "summary": getattr(pkg, "summary", "")
    })
print(json.dumps(packages))
"#;
        
        let output = Command::new(&python_path)
            .args(["-c", script])
            .output()?;
        
        if output.status.success() {
            let json_output = String::from_utf8_lossy(&output.stdout);
            if let Ok(pkg_list) = serde_json::from_str::<Vec<serde_json::Value>>(&json_output) {
                for pkg in pkg_list {
                    if let (Some(name), Some(version)) = (
                        pkg.get("name").and_then(|n| n.as_str()),
                        pkg.get("version").and_then(|v| v.as_str()),
                    ) {
                        packages.push(Package {
                            name: name.to_string(),
                            version: version.to_string(),
                            summary: pkg.get("summary")
                                .and_then(|s| s.as_str())
                                .unwrap_or("")
                                .to_string(),
                        });
                    }
                }
                return Ok(packages);
            }
        }
    }
    
    // If we still couldn't get packages, return an empty list
    Ok(packages)
}

pub fn list_global_packages() -> io::Result<Vec<Package>> {
    let mut packages = Vec::new();
    
    // Try with pip
    let output = Command::new("pip")
        .args(["list", "--format=json"])
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let json_output = String::from_utf8_lossy(&output.stdout);
            if let Ok(pkg_list) = serde_json::from_str::<Vec<serde_json::Value>>(&json_output) {
                for pkg in pkg_list {
                    if let (Some(name), Some(version)) = (
                        pkg.get("name").and_then(|n| n.as_str()),
                        pkg.get("version").and_then(|v| v.as_str()),
                    ) {
                        packages.push(Package {
                            name: name.to_string(),
                            version: version.to_string(),
                            summary: pkg.get("summary")
                                .and_then(|s| s.as_str())
                                .unwrap_or("")
                                .to_string(),
                        });
                    }
                }
            }
            
            return Ok(packages);
        }
    }
    
    // Try with pip3 if pip failed
    let output = Command::new("pip3")
        .args(["list", "--format=json"])
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let json_output = String::from_utf8_lossy(&output.stdout);
            if let Ok(pkg_list) = serde_json::from_str::<Vec<serde_json::Value>>(&json_output) {
                for pkg in pkg_list {
                    if let (Some(name), Some(version)) = (
                        pkg.get("name").and_then(|n| n.as_str()),
                        pkg.get("version").and_then(|v| v.as_str()),
                    ) {
                        packages.push(Package {
                            name: name.to_string(),
                            version: version.to_string(),
                            summary: pkg.get("summary")
                                .and_then(|s| s.as_str())
                                .unwrap_or("")
                                .to_string(),
                        });
                    }
                }
            }
        }
    }
    
    // If both pip and pip3 failed, try using Python directly
    if packages.is_empty() {
        let script = r#"
import sys
import json
import pkg_resources

packages = []
for pkg in pkg_resources.working_set:
    packages.append({
        "name": pkg.project_name,
        "version": pkg.version,
        "summary": getattr(pkg, "summary", "")
    })
print(json.dumps(packages))
"#;
        
        for python_cmd in &["python", "python3"] {
            let output = Command::new(python_cmd)
                .args(["-c", script])
                .output();
            
            if let Ok(output) = output {
                if output.status.success() {
                    let json_output = String::from_utf8_lossy(&output.stdout);
                    if let Ok(pkg_list) = serde_json::from_str::<Vec<serde_json::Value>>(&json_output) {
                        for pkg in pkg_list {
                            if let (Some(name), Some(version)) = (
                                pkg.get("name").and_then(|n| n.as_str()),
                                pkg.get("version").and_then(|v| v.as_str()),
                            ) {
                                packages.push(Package {
                                    name: name.to_string(),
                                    version: version.to_string(),
                                    summary: pkg.get("summary")
                                        .and_then(|s| s.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                });
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
    
    Ok(packages)
}

pub fn create_environment(name: &str) -> io::Result<PythonEnvironment> {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let venv_dir = home_dir.join(".virtualenvs").join(name);
    
    // Create the .virtualenvs directory if it doesn't exist
    let virtualenvs_dir = home_dir.join(".virtualenvs");
    if !virtualenvs_dir.exists() {
        fs::create_dir_all(&virtualenvs_dir)?;
    }
    
    let output = Command::new("python")
        .args(["-m", "venv", venv_dir.to_str().unwrap()])
        .output()?;
    
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create environment: {}", String::from_utf8_lossy(&output.stderr)),
        ));
    }
    
    if let Some(env) = create_environment_from_path(&venv_dir, "venv") {
        Ok(env)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to create environment",
        ))
    }
}

pub fn delete_environment(env_path: &Path) -> io::Result<()> {
    fs::remove_dir_all(env_path)
}

pub fn install_package(env_path: &Path, package_name: &str) -> io::Result<()> {
    // Try to find pip in different locations
    let possible_pip_paths = vec![
        if cfg!(windows) {
            env_path.join("Scripts").join("pip.exe")
        } else {
            env_path.join("bin").join("pip")
        },
        if cfg!(windows) {
            env_path.join("Scripts").join("pip3.exe")
        } else {
            env_path.join("bin").join("pip3")
        },
        // For system Python, try to use the Python executable to run pip as a module
        if cfg!(windows) {
            env_path.join("python.exe")
        } else {
            env_path.join("bin").join("python")
        },
    ];
    
    for pip_path in possible_pip_paths {
        if !pip_path.exists() {
            continue;
        }
        
        // If this is a Python executable, use it to run pip as a module
        let output = if pip_path.file_name().map_or(false, |name| name == "python" || name == "python.exe") {
            Command::new(&pip_path)
                .args(["-m", "pip", "install", package_name])
                .output()
        } else {
            Command::new(&pip_path)
                .args(["install", package_name])
                .output()
        };
        
        match output {
            Ok(output) if output.status.success() => {
                return Ok(());
            },
            Ok(output) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to install package: {}", String::from_utf8_lossy(&output.stderr)),
                ));
            },
            Err(_) => {
                // Try the next pip path
            }
        }
    }
    
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Could not find pip executable",
    ))
}

pub fn uninstall_package(env_path: &Path, package_name: &str) -> io::Result<()> {
    // Try to find pip in different locations
    let possible_pip_paths = vec![
        if cfg!(windows) {
            env_path.join("Scripts").join("pip.exe")
        } else {
            env_path.join("bin").join("pip")
        },
        if cfg!(windows) {
            env_path.join("Scripts").join("pip3.exe")
        } else {
            env_path.join("bin").join("pip3")
        },
        // For system Python, try to use the Python executable to run pip as a module
        if cfg!(windows) {
            env_path.join("python.exe")
        } else {
            env_path.join("bin").join("python")
        },
    ];
    
    for pip_path in possible_pip_paths {
        if !pip_path.exists() {
            continue;
        }
        
        // If this is a Python executable, use it to run pip as a module
        let output = if pip_path.file_name().map_or(false, |name| name == "python" || name == "python.exe") {
            Command::new(&pip_path)
                .args(["-m", "pip", "uninstall", "-y", package_name])
                .output()
        } else {
            Command::new(&pip_path)
                .args(["uninstall", "-y", package_name])
                .output()
        };
        
        match output {
            Ok(output) if output.status.success() => {
                return Ok(());
            },
            Ok(output) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to uninstall package: {}", String::from_utf8_lossy(&output.stderr)),
                ));
            },
            Err(_) => {
                // Try the next pip path
            }
        }
    }
    
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Could not find pip executable",
    ))
}

