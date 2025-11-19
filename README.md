# TypeStorm ⚡

TypeStorm is a lightning-fast, terminal-based typing speed checker written in Rust. It brings the aesthetics and functionality of modern web-based typing tests directly to your CLI.

## 📦 Installation

TypeStorm is available on [crates.io](https://crates.io/crates/typestorm). Install it easily with cargo:

```bash
cargo install typestorm
```

## 🚀 Quick Start

Once installed, simply run:

```bash
typestorm
```

Or run directly from source:
```bash
cargo run
```

## 🎮 How to Use

1.  **Start**: Launch the app. You'll be greeted by the welcome screen. Press `Enter` to begin.
2.  **Type**: Type the words shown on the screen.
    *   **Green**: Correct character.
    *   **Red**: Incorrect character.
    *   **Gray**: Pending character.
3.  **Results**: Once you finish the text, your WPM (Words Per Minute) and Accuracy will be displayed.
4.  **Controls**:
    *   `Esc`: Cancel test / Return to menu.
    *   `Backspace`: Correct mistakes.
    *   `Ctrl+C` or `q`: Quit.

## 🎨 Design Philosophy

TypeStorm was built with three core principles in mind:

### 1. Terminal-First Minimalism
We believe tools should live where developers live: the terminal. TypeStorm provides a distraction-free environment without the bloat of a web browser.

### 2. Performance & Safety
Built with **Rust**, TypeStorm leverages the language's memory safety and speed. It uses `ratatui` for efficient rendering and `crossterm` for cross-platform compatibility, ensuring a smooth experience on any modern terminal.

### 3. Modern Aesthetics
CLI tools don't have to look ancient. We prioritize a clean, colorful, and responsive UI that feels "premium" to use, with immediate visual feedback for every keystroke.

## 🛠️ Tech Stack

*   **Language**: Rust
*   **UI Engine**: [Ratatui](https://github.com/ratatui-org/ratatui)
*   **Terminal Backend**: [Crossterm](https://github.com/crossterm-rs/crossterm)
