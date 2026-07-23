# Autunpad

Desktop multi-tab text editor with integrated script execution.

## Features

- Multi-tabs with drag-and-drop  
- File explorer in the sidebar  
- Script execution (Python, PowerShell, Batch, Shell)  
- Integrated terminal with multiple sessions  
- Syntax highlighting (JavaScript, Python, Markdown, JSON, C, Rust)  
- Find and replace (Ctrl+H)  
- Saved files history  
- Persistent settings (font size, line wrap, line numbers)  

## Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+N   | New tab |
| Ctrl+O   | Open file |
| Ctrl+S   | Save file |
| Ctrl+H   | Find and replace |
| F5       | Run script |

## Technologies

- **Backend**: Rust + Tauri 2  
- **Frontend**: HTML/CSS/JS vanilla  
- **Dialogs**: rfd  
- **Build**: Cargo  

## Development

```bash
# Install Tauri 2 CLI (local)
npm install

# Run in dev mode
npx tauri dev

# Build release
npx tauri build
```

> **Note:** The global `cargo tauri` on your machine may be v1. Use `npx tauri` (v2) for this project.

## Structure

```
src/
  main.rs        # Tauri commands and setup
  file_ops.rs    # File operations
  security.rs    # Path and extension validation
  executor.rs    # Parallel script execution
ui/
  index.html     # Full interface
```
