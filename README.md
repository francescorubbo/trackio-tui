# trackio-tui

A Rust-based Terminal User Interface for visualizing [trackio](https://github.com/gradio-app/trackio) experiments. A keyboard-driven terminal dashboard for browsing your ML experiments.

## Features

- **Full Keyboard Navigation**: Navigate projects, runs, and metrics using vim-style or arrow keys
- **Real-time Updates**: Auto-refresh to monitor live training runs
- **Multi-run Comparison**: Overlay multiple runs on the same chart for comparison
- **Fast & Lightweight**: Native Rust binary with minimal resource usage

## Installation

### Pre-built Binaries

Install with a single command (Linux/macOS):

```bash
curl -sSL https://raw.githubusercontent.com/francescorubbo/trackio-tui/main/install.sh | bash
```

This installs the latest stable release to `~/.local/bin` by default (no sudo required). Make sure it's in your PATH:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

**Install options:**

| Option | Description |
|--------|-------------|
| `--system` | Install to `/usr/local/bin` (requires sudo) |
| `--pre` | Include pre-releases when finding latest version |
| `--version <tag>` | Install a specific version |

**Examples:**

```bash
# Install latest stable release
curl -sSL https://raw.githubusercontent.com/francescorubbo/trackio-tui/main/install.sh | bash

# Install latest pre-release
curl -sSL https://raw.githubusercontent.com/francescorubbo/trackio-tui/main/install.sh | bash -s -- --pre

# Install specific version
curl -sSL https://raw.githubusercontent.com/francescorubbo/trackio-tui/main/install.sh | bash -s -- --version v0.1.0

# System-wide install of a pre-release
curl -sSL https://raw.githubusercontent.com/francescorubbo/trackio-tui/main/install.sh | bash -s -- --pre --system
```

For Windows, download the `.zip` from the [releases page](https://github.com/francescorubbo/trackio-tui/releases).

### Using Cargo

```bash
cargo install trackio-tui
```

### From Source

```bash
git clone https://github.com/francescorubbo/trackio-tui.git
cd trackio-tui
cargo install --path .
```

## Usage

### Basic Usage

```bash
# Launch dashboard (shows all projects)
trackio-tui

# Load a specific project
trackio-tui --project "my-project"

# Custom refresh interval (seconds, default is 2)
trackio-tui --interval 5

# Point to a different database location
trackio-tui --db-path /path/to/trackio/data
```

## Tutorial

This step-by-step guide walks you through using trackio-tui to explore your experiments.

### Prerequisites

Before using trackio-tui, you need experiment data from [trackio](https://github.com/gradio-app/trackio). Run some training experiments first:

```python
import trackio

trackio.init("my-project")
for epoch in range(10):
    trackio.log({"train_loss": 1.0 / (epoch + 1), "accuracy": epoch * 0.1})
```

### Step 1: Launch the Dashboard

Open your terminal and run:

```bash
trackio-tui
```

The dashboard will display all your trackio projects in the left sidebar.

### Step 2: Navigate Projects

Use these keys to browse your projects:

- Press `j` or `↓` to move down the project list
- Press `k` or `↑` to move up
- The selected project's runs appear in the Runs panel below

### Step 3: Browse Runs

Press `Tab` to switch focus to the Runs panel, then:

- Use `j`/`k` or arrow keys to select different runs
- The chart on the right updates to show the selected run's metrics
- Press `Esc` to return focus to the Projects panel

### Step 4: View Different Metrics

The bottom of the chart area shows available metrics numbered `[1]`, `[2]`, etc.

- Press number keys `1`-`9` to focus a metric slot (indicated by `*`)
- Press `Space` to toggle the focused metric for overlay (indicated by `•`)
- Press `Backspace` to clear all overlaid metrics
- When multiple metrics are overlaid, colors differentiate runs and markers differentiate metrics

### Step 5: Compare Multiple Runs

To overlay multiple runs on the same chart for comparison:

1. With the Runs panel focused, navigate to a run you want to compare
2. Press `s` to mark it for comparison (a marker appears next to the run)
3. Navigate to other runs and press `s` to add them
4. All marked runs appear on the chart together with different colors

To clear all comparisons, press `S` (Shift+s).

### Step 6: Monitor Live Training

If you have training runs in progress:

- Data refreshes automatically every 2 seconds (configurable with `--interval`)
- Press `r` to manually refresh at any time

### Step 7: Get Help

Press `h`, `?`, or `F1` at any time to show the help overlay with all keyboard shortcuts.

Press `q` to quit the application.

## UI Layout

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ trackio-tui: my-project                                [?] Help  [q] Quit    │
├───────────────────┬──────────────────────────────────────────────────────────┤
│                   │                                                          │
│ Projects          │  Metrics: train_loss                                     │
│ ───────────────   │  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~                │
│ ▶ my-project      │  1.0 │-                                                  │
│   other-project   │      │ -                                                 │
│                   │      │  --                                               │
│ Runs (3)          │      │    --                                             │
│ ───────────────   │  0.5 │      ---                                          │
│ ▶ run-001         │      │         -----                                     │
│   run-002         │      │              -------                              │
│ ● run-003         │  0.0 └──────────────────────────────────────────▶ step   │
│                   │      0                                        100        │
│ Config            │                                                          │
│ ───────────────   │  [1] train_loss  [2] val_loss  [3] accuracy              │
│ epochs: 10        │                                                          │
│ lr: 0.001         │                                                          │
│ batch: 64         │                                                          │
└───────────────────┴──────────────────────────────────────────────────────────┘
```

## Data Source

trackio-tui reads from trackio's SQLite database, located by default at:

```
~/.cache/huggingface/trackio/
```

You can override this with:
- The `--db-path` CLI argument
- The `TRACKIO_DIR` environment variable

## Requirements

- A terminal with 256-color or true-color support
- trackio experiment data (run some experiments first!)

## Comparison with trackio-view

| Feature | trackio-tui (this) | trackio-view |
|---------|-------------------|--------------|
| Language | Rust | Python |
| Installation | Single binary | pip install |
| GPU Monitoring | No | Yes |
| Dependencies | None | Python + trackio |

For GPU monitoring features, consider using [trackio-view](https://github.com/mcgrof/trackio-view).

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Acknowledgments

- [trackio](https://github.com/gradio-app/trackio) - The experiment tracking library
- [ratatui](https://github.com/ratatui/ratatui) - The Rust TUI framework
- [trackio-view](https://github.com/mcgrof/trackio-view) - Inspiration for terminal visualization

