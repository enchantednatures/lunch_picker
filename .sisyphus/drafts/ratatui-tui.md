# Draft: Ratatui TUI for lunch_picker

## Requirements (confirmed)
- **Scope**: Subcommand / Side-by-side with existing CLI. Add a `tui` command to launch it.
- **Features**: Everything. Include all existing features: Homies, Restaurants, Recipes, Meal Plans, Pantry, Shopping List, Templates, and the Lunch Picker.
- **Navigation**: User wants a recommendation. Prometheus will suggest a sidebar + main content layout.

## Technical Decisions
- **Library**: `ratatui` (user-requested).
- **Backend**: Existing SQLite + sqlx + tokio runtime.
- **Current UI**: Uses `dialoguer` for interactive prompts. TUI will replace these interactive flows.
- **App State**: Currently `AppState { db: Pool<Sqlite> }`. TUI will need its own state management (ratatui app state).

## Research Findings
- **Project Structure**: `src/main.rs` is a large CLI dispatch block. `src/interaction.rs` contains dialoguer-based prompts.
- **Features**: Modular under `src/features/`. Each feature has DB traits and functions.
- **Async**: Heavily async (tokio). TUI will need to bridge sync crossterm events with async DB calls.
- **Config**: JSON config file, telemetry setup, DB setup all in `main.rs`.

## Open Questions
- **Design**: Any color/theme preferences? (Default ratatui style for now)

## Decisions Made
- **Testing**: Add unit tests for TUI state/logic (App state transitions, event handling).
- **Keybindings**: Vim-style (hjkl navigation, q to quit, / to search, etc.) with arrow key fallbacks.
- **Navigation Pattern**: Sidebar (left) + Main Content (right). Sidebar lists sections: Dashboard, Pick, Homies, Restaurants, Recipes, Meal Plan, Pantry, Shopping List, Templates.
- **Async Architecture**: Use a tokio `mpsc` channel to dispatch async DB operations from the TUI event loop. The TUI runs on the main thread with crossterm, spawning tokio tasks for DB calls that send results back via channel.

## Scope Boundaries
- **INCLUDE**: All existing features exposed via TUI.
- **EXCLUDE**: Removing existing CLI args (they stay). Changing DB schema (unless needed for TUI state).
