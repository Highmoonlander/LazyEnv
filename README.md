# 🐢 LazyEnv

**LazyEnv** is a fast and intuitive terminal-based Python environment manager that brings the simplicity of GUI tools to your command line.

> Manage all your virtual environments and packages with zero clutter — just clean TUI magic. ✨

---

## 📌 Overview

LazyEnv is built in **Rust** and designed for developers who juggle multiple Python projects. It helps you:

- Detect and manage Python environments (`venv`, `conda`, `pyenv`, etc.)
- Create and delete virtual environments
- Install, remove, and view Python packages
- Quickly search and navigate via keyboard
- Work fully within your terminal

---

## 🧰 Features

| Feature                            | Description                                                                 |
|------------------------------------|-----------------------------------------------------------------------------|
| 🔎 Auto Detection                  | Finds all Python environments on your system                               |
| 🌱 Create/Delete                   | Easily spin up or remove environments                                      |
| 📦 Package Manager                 | Install/uninstall packages in any selected environment                     |
| 🎯 Focused Navigation              | Keyboard-driven UI with smooth focus switching                             |
| 🌐 Global vs Env Packages          | Toggle between environment-specific and global package views               |
| 🖥️ Built-in Terminal UI            | Clean and responsive interface powered by Ratatui + Crossterm              |

---

## ⚙️ Installation

LazyEnv is published as a global CLI tool.

```bash
npm install -g @highmoonlander/lazyenv@0.1.2
```

## 🚀 Getting Started
```bash
lazyenv
```
## 🎮 Keyboard Controls

### General
	•	↑ / ↓ — Move selection
	•	Tab — Switch focus (envs <-> packages)
	•	Enter — View packages in selected environment
	•	Esc — Cancel or go back
	•	q — Quit application
	•	x — Toggle help menu

### Environment Actions
	•	n — Create new environment
	•	d — Delete selected environment
	•	s — Search environments
	•	R — Refresh environment list

Package Actions
	•	i — Install new package
	•	r — Remove selected package
	•	g — Toggle global/environment packages

## 🔍 Environment Detection

LazyEnv automatically detects environments from:
	•	System Python
	•	Local .venv/ folders
	•	~/.virtualenvs/
	•	~/.venv/
	•	~/.pyenv/versions/
	•	Conda environments

## 🤝 Contributing
	1.	Fork the repo
	2.	Create a branch: git checkout -b feature/your-feature
	3.	Commit changes: git commit -m "feat: your message"
	4.	Push: git push origin feature/your-feature
	5.	Open a Pull Request

Issues and suggestions are welcome too!

## 📜 License

Licensed under the MIT License.