# trackio-tui

A Rust-based Terminal User Interface for visualizing [trackio](https://github.com/gradio-app/trackio) experiments. This is a drop-in replacement for `trackio show` providing a keyboard-driven terminal dashboard.

![trackio-tui demo](https://via.placeholder.com/800x400?text=trackio-tui+Dashboard)

## Features

- **Full Keyboard Navigation**: Navigate projects, runs, and metrics using vim-style or arrow keys
- **Real-time Updates**: Auto-refresh to monitor live training runs
- **Multi-run Comparison**: Overlay multiple runs on the same chart for comparison
- **Smoothing Controls**: Apply exponential moving average to smooth noisy metrics
- **X-axis Zoom**: Focus on specific portions of training history
- **Theme Support**: Choose from built-in themes or customize colors
- **Fast & Lightweight**: Native Rust binary with minimal resource usage

## Installation

### Pre-built Binaries

Install with a single command (Linux/macOS):

```bash
curl -sSL https://raw.githubusercontent.com/francescorubbo/trackio-tui/main/install.sh | bash
```

This installs to `~/.local/bin` by default (no sudo required). Make sure it's in your PATH:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

To install system-wide to `/usr/local/bin` instead:

```bash
curl -sSL https://raw.githubusercontent.com/francescorubbo/trackio-tui/main/install.sh | bash -s -- --system
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
trackio-tui show

# Load a specific project
trackio-tui show --project "my-project"

# With a different theme
trackio-tui show --theme "soft"

# With custom colors
trackio-tui show --color-palette "#FF6B6B,#4ECDC4,#45B7D1"

# Custom refresh interval (seconds)
trackio-tui show --interval 5

# Point to a different database location
trackio-tui show --db-path /path/to/trackio/data
```

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` / `l` | Select / expand |
| `Esc` / `h` | Go back / collapse |
| `Tab` | Cycle panels (Projects → Runs → Metrics) |
| `Shift+Tab` | Cycle panels backwards |

### Metrics

| Key | Action |
|-----|--------|
| `1-9` | Quick-select metric by number |
| `+` / `-` | Adjust smoothing |
| `[` / `]` | Adjust x-axis range (zoom) |

### Run Comparison

| Key | Action |
|-----|--------|
| `s` | Toggle run for comparison |
| `S` | Clear all comparison selections |

### Other

| Key | Action |
|-----|--------|
| `r` | Refresh data |
| `?` / `F1` | Toggle help overlay |
| `q` | Quit |

## UI Layout

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ trackio-tui: my-project                                [?] Help  [q] Quit    │
├───────────────────┬──────────────────────────────────────────────────────────┤
│                   │                                                          │
│ Projects          │  Metrics: train_loss                                     │
│ ───────────────   │  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~                │
│ ▶ my-project      │  1.0 │⣿                                                  │
│   other-project   │      │ ⣿                                                 │
│                   │      │  ⣿⣿                                               │
│ Runs (3)          │      │    ⣿⣿                                             │
│ ───────────────   │  0.5 │      ⣿⣿⣿                                          │
│ ▶ run-001 [done]  │      │         ⣿⣿⣿⣿                                      │
│   run-002 [done]  │      │              ⣿⣿⣿⣿⣿⣿                               │
│ ● run-003 [live]  │  0.0 └──────────────────────────────────────────▶ step   │
│                   │      0                                        100        │
│ Config            │                                                          │
│ ───────────────   │  [1] train_loss  [2] val_loss  [3] accuracy              │
│ epochs: 10        │                                                          │
│ lr: 0.001         ├──────────────────────────────────────────────────────────┤
│ batch: 64         │  Smoothing: [=====               ] 5                     │
└───────────────────┴──────────────────────────────────────────────────────────┘
```

## Themes

Built-in themes:

- `default` - Standard dark theme with vibrant colors
- `soft` - Muted colors, easier on the eyes
- `dark` - High contrast dark theme
- `light` - Light background theme

Custom color palettes can be specified with `--color-palette` using comma-separated hex colors.

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
| Startup Time | Fast | Slower |
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

