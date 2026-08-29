# Termiban — Terminal Kanban

**Self-hosted personal Kanban for your terminal. Linux-only.**

No cloud, no account. Your board lives as a single `board.json` on your machine. Manage tasks with deadlines, priorities, and tags — all from a fast TUI.

Built with Rust, `ratatui` + `crossterm`. Clean, modular codebase.

---

## Why Termiban?

- **Private by design** — local JSON file, no sync required
- **Terminal-native** — stays in your workflow, works over SSH
- **Custom boards** — any columns you want, not just To Do / Doing / Done
- **Rich tasks** — title + description, due dates, priority, tags
- **Start from scratch** — delete everything and rebuild; onboarding guides you

---

## Features

**Boards**
- Add, rename, delete, and reorder columns
- Start empty and build your own flow, or create from template
- Welcome screen when empty guides you to your first column

**Tasks**
- **Title** and **description** — description is *long* (wraps, no limit, shown full in detail view)
- **Deadline** (`YYYY-MM-DD`) — overdue in red, today in yellow
- **Priority** — Low · Medium ▲ High ‼ Urgent (color + icon, `p` to cycle)
- **Tags** — comma-separated, e.g. `work, bug`
- **URL** — `https://…` (`🔗` in list, `o` to open in browser, `https://` required)

All changes auto-save.

---

## Install

Requires Rust 1.98+.

```bash
git clone https://github.com/revprm/termiban
cd termiban
cargo build --release
# binary at target/release/termiban

# optional: install to ~/.cargo/bin
cargo install --path .

# run
cargo run --release
# or
termiban
```

Portable data file:

```bash
TERMIBAN_DATA_PATH=/tmp/board.json termiban
```

**Compatibility:** Linux only. macOS and Windows are not supported.

---

## Quick Start

```bash
termiban
```

1. **First run** gives you a template board: `To Do` → `In Progress` → `Done`.
2. Press `?` for help, `q` to quit.

**To build your own board from scratch:**

- Open board manager: `B`
- Press `c` to clear, then `a` to add `Backlog`, `C` to add `Doing`, `C` to add `Review`…

Or delete columns one by one with `B` → `d` until empty — the welcome screen will offer `a` (add column) or `t` (template).

---

## Keybindings

**Navigate**
- `h`/`l` or `←`/`→` — columns
- `j`/`k` or `↓`/`↑` — tasks
- `H`/`L` — move task to prev/next column

**Tasks**
- `n` — quick add (title only) · `N` — full add (all fields incl. URL + long desc)
- `e` — quick edit · `E` — full edit
- `Enter` / `v` — view details (wrapped long description + URL)
- `p` — cycle priority · `o` — open URL
- `d` / `x` — delete

**In full form (`N`/`E`) — 6 fields: Title → Description (long) → Deadline → Priority → Tags → URL**
- `Tab` / `Shift+Tab` — next/prev field
- `p` on Priority — cycle
- `Enter` — save (validates `YYYY-MM-DD` and `https://`), `Esc` — cancel

**Board**
- `B` — board manager → `a` add, `r` rename, `d` delete, `t` template, `c` clear, `H`/`L` move
- `C` — quick add column, `R` — quick rename

**General:** `?` help, `q`/`Esc` quit.

---

## Where is my data?

Auto-saved JSON. Location (in order):

1. `$TERMIBAN_DATA_PATH` or `$TERMIBAN_DATA` if set
2. `$XDG_DATA_HOME/termiban/board.json`
3. `~/.local/share/termiban/board.json`
4. `./board.json`

Long descriptions? Paste as much as you want — detail view wraps, no hard limit. Add a URL for quick `o` open.

Edit `board.json` by hand if you like:

```json
{
  "title": "My Board",
  "columns": [
    {"name": "Backlog", "tasks": []},
    {"name": "Doing", "tasks": [
      {"id":1,"title":"Fix login","description":"Long notes here...\nMultiple paragraphs wrap in detail view.","deadline":"2026-09-01","priority":"high","tags":["work"],"url":"https://github.com/me/proj","created_at":"2026-08-30"}
    ]}
  ],
  "next_id": 2
}
```

---

## Project Structure

Highly modular — easy to read and extend:

```
src/
├── main.rs        — terminal setup
├── board/         — data model (Task, Column, Board, Priority)
├── storage.rs     — load/save
├── app/           — state, task & board logic, key handlers
└── ui/            — rendering (board, footer, popups)
    └── popups/    — welcome, task form/detail, board manager, help
```

- `board` is pure data, `storage` is only I/O
- `app` handles all logic, `ui` only reads state

---

## Development

```bash
cargo check
cargo test -- --test-threads=1   # 14 tests
cargo build --release
```

---

## License

MIT
