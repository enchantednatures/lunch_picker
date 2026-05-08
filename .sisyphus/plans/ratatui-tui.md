# Ratatui TUI for lunch_picker

## TL;DR
> Build a full Terminal User Interface (TUI) for `lunch_picker` using `ratatui` + `crossterm`, launched via a new `tui` CLI subcommand. The TUI runs side-by-side with the existing CLI and exposes all features: Homies, Restaurants, Recipes, Meal Plans, Pantry, Shopping List, Templates, and the Lunch Picker.
>
> **Deliverables**:
> - New `src/tui/` module directory with app state, event loop, UI widgets, and views.
> - `ratatui`, `crossterm` added to `Cargo.toml`.
> - New `Command::Tui` variant in `src/cli_args.rs`.
> - Unit tests for TUI state transitions and event handling.
> - Vim-style keybindings (hjkl, q) with arrow-key fallbacks.
> - Sidebar + Main Content navigation pattern.
>
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 3 waves
> **Critical Path**: T1 (scaffold) → T2 (event loop) → T7 (integration) → F1-F4

---

## Context

### Original Request
"i want to plan a tui for this using ratatui"

### Interview Summary
**Key Discussions**:
- **Scope**: Subcommand / Side-by-side. Keep existing CLI intact; add `lunch_picker tui` to launch.
- **Features**: Everything. All existing features must be accessible in the TUI.
- **Navigation**: User deferred to recommendation. Selected **Sidebar + Main Content** layout.
- **Keybindings**: Vim-style (hjkl, q to quit, / to search) with arrow-key fallbacks.
- **Testing**: Unit tests for TUI state/logic (App state transitions, event handling).
- **Async**: Bridge sync crossterm event loop with async sqlx DB calls via tokio `mpsc` channel.

**Research Findings**:
- Project is a Rust CLI using `clap`, `dialoguer`, `sqlx` (SQLite), `tokio`.
- `src/main.rs` is a large CLI dispatch block (~583 lines). `src/interaction.rs` has `dialoguer`-based prompts that the TUI will supersede.
- Features are modular under `src/features/` with DB traits.
- App state (`AppState`) currently just holds `db: Pool<Sqlite>`.
- Existing CLI supports: Homies (add/delete/rename/favorites/recent), Restaurants (add/delete/rename), Recipes (add/delete/view/list/import/steps/ingredients), Pantry (add/update/remove/list), Meal Plan (add/remove/view/shop), Templates (add/remove/list/apply), and the `Pick` flow.

### Metis Review
**Identified Gaps** (addressed):
- **Gap**: Async TUI pattern not specified.
  - **Resolution**: Use tokio `mpsc` channel. TUI event loop runs on main thread (crossterm). Async DB tasks spawn via `tokio::spawn`, sending results back through channel to update App state.
- **Gap**: Error handling in TUI not defined.
  - **Resolution**: Toast/notification bar at bottom of screen for transient errors. Modals for destructive actions (delete confirmation).
- **Gap**: Input method for adding items (homies, restaurants, etc.).
  - **Resolution**: Inline forms within the main content area using ratatui `Paragraph` + `Block` + key capture. Press `i` to insert (vim-style) on list views.
- **Gap**: Scrolling for long lists.
  - **Resolution**: `List` widget with scroll offset state. `j/k` or `↓/↑` to scroll, `PgUp/PgDn` for page jumps.
- **Gap**: Feature parity breakdown vague.
  - **Resolution**: Explicitly map each CLI subcommand to a TUI view in TODOs.

---

## Work Objectives

### Core Objective
Implement a complete ratatui-based TUI as a first-class interface for `lunch_picker`, maintaining full feature parity with the existing CLI while providing a more intuitive, interactive terminal experience.

### Concrete Deliverables
- `Cargo.toml` updated with `ratatui`, `crossterm` dependencies.
- `src/tui/` module directory containing:
  - `mod.rs` — module exports
  - `app.rs` — main App state struct, current view, message queue, quit flag
  - `event.rs` — crossterm event reader (keyboard, resize, mouse optional)
  - `ui.rs` — root render function, layout constraints (sidebar + main + footer)
  - `widgets/` — reusable widgets (sidebar nav, notification bar, input form, confirmation modal)
  - `views/` — one module per screen (dashboard, pick, homies, restaurants, recipes, meal_plan, pantry, shopping_list, templates)
- `src/cli_args.rs` updated with `Command::Tui`.
- `src/main.rs` updated to dispatch `Command::Tui`.
- Unit tests in `src/tui/` for state transitions.

### Definition of Done
- [ ] `cargo run -- tui` launches the TUI successfully.
- [ ] All sidebar sections are navigable and render data from the DB.
- [ ] The `Pick` flow works end-to-end within the TUI.
- [ ] Unit tests pass (`cargo test`).
- [ ] Existing CLI commands continue to work unchanged.

### Must Have
- Sidebar navigation with all feature sections.
- Vim-style keybindings (hjkl, q, /) + arrow keys.
- Async DB operations do not block the UI.
- Error notifications display in the UI.
- Confirmation modal before delete operations.
- Scrollable lists for all collections.
- Inline forms for adding/editing items.

### Must NOT Have (Guardrails)
- Do NOT remove or refactor existing CLI argument parsing (keep `cli_args.rs` intact except for adding `Tui`).
- Do NOT change the DB schema unless absolutely necessary for TUI state persistence (not needed for MVP).
- Do NOT add mouse support in Wave 1 (optional future enhancement).
- Do NOT rewrite feature logic in `src/features/` — TUI is a consumer of existing traits/functions.
- Avoid over-abstraction: keep widgets concrete and tied to domain, not generic UI frameworks.

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (standard `cargo test`)
- **Automated tests**: Tests after (unit tests for TUI logic after implementation)
- **Framework**: Built-in Rust test harness (`#[test]`)
- **If Tests after**: Each task that introduces state logic will include a follow-up test task.

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **TUI/CLI**: Use `interactive_bash` (tmux) — Run command, send keystrokes, validate terminal output.
- **Library/Module**: Use `Bash` (cargo test) — Run tests, assert PASS.
- **Build**: Use `Bash` (cargo build) — Assert zero errors/warnings.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation + Scaffolding — can start immediately):
├── T1: Add ratatui + crossterm deps, create src/tui/ module skeleton [quick]
├── T2: Implement async-safe event loop (crossterm → mpsc → tokio) [deep]
├── T3: Create root App state and UI layout (sidebar + main + footer) [visual-engineering]
├── T4: Build reusable widgets (sidebar nav, notification bar, input form, modal) [visual-engineering]
└── T5: Add Command::Tui variant and main.rs dispatch [quick]

Wave 2 (Views + State Logic — depends on Wave 1):
├── T6: Dashboard view (summary stats: #homies, #restaurants, #recipes, #plans) [quick]
├── T7: Homies view (list, add, delete, rename, manage favorites) [unspecified-high]
├── T8: Restaurants view (list, add, delete, rename) [quick]
├── T9: Recipes view (list, add, delete, view details, import URL) [unspecified-high]
├── T10: Meal Plan view (list, add, remove, view range, generate shopping list) [unspecified-high]
├── T11: Pantry view (list, add, update, remove) [quick]
├── T12: Shopping List view (display generated list) [quick]
├── T13: Templates view (list, add, remove, apply) [quick]
├── T14: Pick flow view (select homies → pick restaurant → show result) [deep]
└── T15: TUI state/logic unit tests [unspecified-high]

Wave 3 (Integration + Polish — depends on Wave 2):
├── T16: Wire all views into sidebar navigation + keybindings [quick]
├── T17: End-to-end QA: launch TUI, navigate all views, perform CRUD [unspecified-high]
├── T18: Final code quality review (clippy, formatting) [quick]
└── T19: Update README with TUI usage instructions [writing]

Wave FINAL (After ALL tasks — 4 parallel reviews, then user okay):
├── F1: Plan compliance audit (oracle)
├── F2: Code quality review (unspecified-high)
├── F3: Real manual QA (unspecified-high)
└── F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: T1 → T2 → T3 → T4 → T7 → T14 → T16 → T17 → F1-F4 → user okay
Parallel Speedup: ~60% faster than sequential
Max Concurrent: 5 (Wave 1) / 10 (Wave 2) / 4 (Wave 3)
```

### Dependency Matrix

| Task | Depends On | Blocks |
|------|-----------|--------|
| T1 | — | T2, T3, T4, T5 |
| T2 | T1 | T3, T7, T9, T10, T14 |
| T3 | T1, T2 | T4, T6, T16 |
| T4 | T1, T3 | T6, T7, T8, T9, T10, T11, T12, T13, T14 |
| T5 | T1 | T16 |
| T6 | T3, T4 | T16 |
| T7 | T2, T4 | T16 |
| T8 | T4 | T16 |
| T9 | T2, T4 | T16 |
| T10 | T2, T4 | T16 |
| T11 | T4 | T16 |
| T12 | T4 | T16 |
| T13 | T4 | T16 |
| T14 | T2, T4 | T16 |
| T15 | T7, T9, T10, T14 | T17 |
| T16 | T5, T6, T7, T8, T9, T10, T11, T12, T13, T14 | T17 |
| T17 | T15, T16 | T18, T19 |
| T18 | T17 | — |
| T19 | T17 | — |

### Agent Dispatch Summary

- **Wave 1**: T1 → `quick`, T2 → `deep`, T3 → `visual-engineering`, T4 → `visual-engineering`, T5 → `quick`
- **Wave 2**: T6 → `quick`, T7 → `unspecified-high`, T8 → `quick`, T9 → `unspecified-high`, T10 → `unspecified-high`, T11 → `quick`, T12 → `quick`, T13 → `quick`, T14 → `deep`, T15 → `unspecified-high`
- **Wave 3**: T16 → `quick`, T17 → `unspecified-high`, T18 → `quick`, T19 → `writing`
- **FINAL**: F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] T1. **Add ratatui + crossterm deps, create `src/tui/` module skeleton**

  **What to do**:
  - Add `ratatui = "0.26"` and `crossterm = "0.27"` to `Cargo.toml` `[dependencies]`.
  - Create `src/tui/mod.rs` with module declarations for `app`, `event`, `ui`, `widgets`, `views`.
  - Create empty placeholder files: `src/tui/app.rs`, `src/tui/event.rs`, `src/tui/ui.rs`.
  - Create `src/tui/widgets/mod.rs` and `src/tui/views/mod.rs`.
  - Add `pub mod tui;` to `src/lib.rs`.

  **Must NOT do**:
  - Do NOT implement any logic yet — just scaffolding.
  - Do NOT remove any existing dependencies.

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Straightforward file creation and Cargo.toml edits.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with T2, T3, T4, T5)
  - **Blocks**: T2, T3, T4, T5
  - **Blocked By**: None

  **References**:
  - `Cargo.toml` — Add deps alongside existing ones (follow semver patterns).
  - `src/lib.rs` — Follow existing `pub mod` pattern.

  **Acceptance Criteria**:
  - [ ] `cargo check` passes with new deps (may need minimal stub content in `.rs` files to compile).
  - [ ] `src/tui/` directory exists with all listed files.

  **QA Scenarios**:
  ```
  Scenario: Dependencies compile
    Tool: Bash
    Preconditions: Clean working directory
    Steps:
      1. Run `cargo check`
    Expected Result: Compilation succeeds, no errors from new deps.
    Evidence: .sisyphus/evidence/t1-compile.log
  ```

  **Commit**: YES (groups with Wave 1)
  - Message: `feat(tui): add ratatui and crossterm dependencies, scaffold tui module`
  - Files: `Cargo.toml`, `src/lib.rs`, `src/tui/*`

- [ ] T2. **Implement async-safe event loop (crossterm → mpsc → tokio)**

  **What to do**:
  - Implement `src/tui/event.rs` with a `EventHandler` that reads `crossterm::event::Event` in a blocking loop on a dedicated thread.
  - Use `crossterm::event::read()` (blocking) and send events through a `std::sync::mpsc::channel` (or `tokio::sync::mpsc`) to the main TUI loop.
  - The main TUI loop (in `app.rs`) runs on the tokio runtime (since `main` is `#[tokio::main]`). It should `select!` between:
    - Receiving crossterm events from the channel.
    - Receiving async DB operation results from a tokio `mpsc` channel.
  - Map crossterm `KeyEvent`s to a custom `AppEvent` enum (e.g., `Quit`, `NavigateUp`, `NavigateDown`, `Select`, `Back`, `InsertMode`, `Submit`, `Search`, etc.).

  **Must NOT do**:
  - Do NOT use async crossterm event readers (keep it simple: blocking thread + channel).
  - Do NOT implement view-specific event handling here — just the generic event loop and mapping.

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Requires careful threading and async/sync boundary handling.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T1, but needs T1 scaffolding)
  - **Parallel Group**: Wave 1
  - **Blocks**: T3, T7, T9, T10, T14
  - **Blocked By**: T1

  **References**:
  - `src/main.rs:122` — `#[tokio::main]` runtime context.
  - Ratatui examples (official repo) — `tokio-demo` or `async` examples for event loop patterns.
  - `crossterm` docs — `event::read()`, `KeyCode`, `KeyModifiers`.

  **Acceptance Criteria**:
  - [ ] `cargo check` passes.
  - [ ] Event loop compiles and can be instantiated in a test.

  **QA Scenarios**:
  ```
  Scenario: Event loop compiles and runs
    Tool: Bash
    Preconditions: T1 complete
    Steps:
      1. Run `cargo test --lib tui::event` (or add a basic instantiation test).
    Expected Result: Test compiles and passes.
    Evidence: .sisyphus/evidence/t2-event-loop-test.log
  ```

  **Commit**: YES (groups with Wave 1)

- [ ] T3. **Create root App state and UI layout (sidebar + main + footer)**

  **What to do**:
  - Implement `src/tui/app.rs`:
    - `struct App` holding: `current_view: View`, `sidebar_selected: usize`, `should_quit: bool`, `notification: Option<String>`, `db: Pool<Sqlite>`.
    - `enum View` with variants for each section: `Dashboard`, `Pick`, `Homies`, `Restaurants`, `Recipes`, `MealPlan`, `Pantry`, `ShoppingList`, `Templates`.
    - `impl App { fn new(db: Pool<Sqlite>) -> Self`, `fn on_event(&mut self, event: AppEvent)`, `fn draw(&mut self, frame: &mut Frame)` }.
  - Implement `src/tui/ui.rs`:
    - `fn ui(frame: &mut Frame, app: &mut App)` that sets up `Layout` with:
      - Left sidebar (20% width) — `Block::bordered("Navigation")`.
      - Main content area (remaining width) — `Block::bordered("Content")`.
      - Bottom footer (3 lines) — `Block::bordered("Status / Notifications")`.
    - Render sidebar list of view names with highlight based on `app.sidebar_selected`.
    - Render footer with current keybind hints (`q: quit`, `j/k: navigate`, `Enter: select`).

  **Must NOT do**:
  - Do NOT render view-specific content in `ui.rs` — dispatch to view renderers.
  - Do NOT handle DB calls directly in `draw` — keep rendering pure.

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Layout and visual structure are the primary concern.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T1, T2)
  - **Parallel Group**: Wave 1
  - **Blocks**: T4, T6, T16
  - **Blocked By**: T1, T2

  **References**:
  - Ratatui docs — `Frame`, `Layout`, `Constraint`, `Direction`, `Block`, `Borders`.
  - `src/main.rs:73-80` — `AppState` pattern to follow for holding `db`.

  **Acceptance Criteria**:
  - [ ] `cargo check` passes.
  - [ ] `App::new` can be constructed in a test.
  - [ ] `App::draw` can be called with a `TestBackend` (ratatui testing backend) without panicking.

  **QA Scenarios**:
  ```
  Scenario: App renders basic layout
    Tool: Bash (cargo test)
    Preconditions: T1, T2 complete
    Steps:
      1. Write a test that creates `App`, calls `draw` with `TestBackend` (80x24).
      2. Assert that rendered buffer contains "Navigation" and "Status" text.
    Expected Result: Test passes.
    Evidence: .sisyphus/evidence/t3-layout-test.log
  ```

  **Commit**: YES (groups with Wave 1)

- [ ] T4. **Build reusable widgets (sidebar nav, notification bar, input form, modal)**

  **What to do**:
  - `src/tui/widgets/sidebar.rs`: `fn render_sidebar(frame: &mut Frame, area: Rect, selected: usize, items: &[&str])` — uses `List` widget with highlight style.
  - `src/tui/widgets/notification_bar.rs`: `fn render_notification(frame: &mut Frame, area: Rect, message: &str, level: NotificationLevel)` — colored bar (red for error, yellow for warn, green for info).
  - `src/tui/widgets/input_form.rs`: `struct InputForm { label: String, value: String, focused: bool }` — renders a label + `Paragraph` with cursor. Handles `Char`, `Backspace`, `Enter`.
  - `src/tui/widgets/modal.rs`: `fn render_modal(frame: &mut Frame, title: &str, message: &str, options: &[&str], selected: usize)` — centered popup block with `Clear` background.

  **Must NOT do**:
  - Do NOT make widgets overly generic (e.g., a generic "Form" that accepts any fields). Keep them concrete for domain use.
  - Do NOT implement business logic in widgets — pure rendering only.

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Custom widgets are visual components.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T1-T3)
  - **Parallel Group**: Wave 1
  - **Blocks**: T6, T7, T8, T9, T10, T11, T12, T13, T14
  - **Blocked By**: T1, T3

  **References**:
  - Ratatui docs — `List`, `Paragraph`, `Block`, `Clear`, `Borders`, `Style`, `Color`.
  - Ratatui examples — `popup`, `user_input`, `list`.

  **Acceptance Criteria**:
  - [ ] Each widget has a basic render test using `TestBackend`.
  - [ ] `cargo test` passes for widget tests.

  **QA Scenarios**:
  ```
  Scenario: Widgets render without panic
    Tool: Bash (cargo test)
    Preconditions: T1-T3 complete
    Steps:
      1. Run widget tests: `cargo test --lib tui::widgets`
    Expected Result: All widget tests pass.
    Evidence: .sisyphus/evidence/t4-widgets-test.log
  ```

  **Commit**: YES (groups with Wave 1)

- [ ] T5. **Add `Command::Tui` variant and `main.rs` dispatch**

  **What to do**:
  - In `src/cli_args.rs`, add `Tui` variant to `Command` enum (no args needed).
  - In `src/main.rs`, add `Command::Tui => { launch_tui(app_state.db).await?; }` in the match block.
  - Create `src/tui/mod.rs` export `pub async fn launch_tui(db: Pool<Sqlite>) -> Result<()>` that:
    - Enables raw mode (`crossterm::terminal::enable_raw_mode`).
    - Creates `Terminal::new(CrosstermBackend::new(stdout))`.
    - Instantiates `App::new(db)`.
    - Runs the event loop (from T2) and draw loop (from T3).
    - On quit, restores terminal (`disable_raw_mode`, `LeaveAlternateScreen`).
  - Use `std::panic::set_hook` to ensure terminal is restored on panic.

  **Must NOT do**:
  - Do NOT change existing CLI command handling.
  - Do NOT make `tui` the default behavior — only when `Command::Tui` is matched.

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Wiring existing pieces together.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T1-T4)
  - **Parallel Group**: Wave 1
  - **Blocks**: T16
  - **Blocked By**: T1

  **References**:
  - `src/cli_args.rs` — Follow existing enum variant patterns.
  - `src/main.rs:180` — Match block location for new command.
  - Ratatui examples — `main.rs` in any example for terminal setup/teardown.

  **Acceptance Criteria**:
  - [ ] `cargo run -- tui` launches and shows the TUI (even if empty).
  - [ ] Pressing `q` quits and returns to normal shell.
  - [ ] Terminal state is restored correctly (no garbled shell after exit).

  **QA Scenarios**:
  ```
  Scenario: TUI launches and quits cleanly
    Tool: interactive_bash (tmux)
    Preconditions: T1-T4 complete
    Steps:
      1. Start tmux session.
      2. Run `cargo run -- tui`.
      3. Wait 2s, send `q` keystroke.
      4. Capture pane output.
    Expected Result: TUI renders, then exits. Shell prompt is clean and functional.
    Failure Indicators: Garbled text, cursor missing, raw mode stuck.
    Evidence: .sisyphus/evidence/t5-launch-quit.log
  ```

  **Commit**: YES (groups with Wave 1)

- [ ] T6. **Dashboard view (summary stats)**

  **What to do**:
  - Create `src/tui/views/dashboard.rs`.
  - Render a grid of summary cards showing:
    - Total Homies (count from `get_all_homies`).
    - Total Restaurants (count from `get_all_restaurants`).
    - Total Recipes (count from `list_recipes`).
    - Upcoming Meal Plan entries (next 7 days from `list_meal_plan_entries`).
  - Fetch counts asynchronously via the tokio channel pattern (spawn task, send result back).
  - Store counts in `App` state (e.g., `dashboard_counts: Option<DashboardCounts>`).

  **Must NOT do**:
  - Do NOT block the UI waiting for DB queries.
  - Do NOT add edit functionality — this is read-only summary.

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Read-only view with simple async fetches.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T7-T14)
  - **Parallel Group**: Wave 2
  - **Blocks**: T16
  - **Blocked By**: T3, T4

  **References**:
  - `src/features/homies.rs` — `get_all_homies` signature.
  - `src/features/restaurants.rs` — `get_all_restaurants` signature.
  - `src/features/recipes.rs` — `list_recipes` signature.
  - `src/features/meal_plan_entries.rs` — `list_meal_plan_entries` signature.
  - Ratatui `Table` or `Paragraph` widgets for cards.

  **Acceptance Criteria**:
  - [ ] Dashboard renders without panic.
  - [ ] Counts load asynchronously and display.
  - [ ] Unit test: `DashboardView::render` with mock state.

  **QA Scenarios**:
  ```
  Scenario: Dashboard loads data
    Tool: Bash (cargo test)
    Preconditions: T1-T5 complete
    Steps:
      1. Run dashboard unit test.
    Expected Result: Test passes, mock counts render correctly.
    Evidence: .sisyphus/evidence/t6-dashboard-test.log
  ```

  **Commit**: YES (groups with Wave 2)

- [ ] T7. **Homies view (list, add, delete, rename, manage favorites)**

  **What to do**:
  - Create `src/tui/views/homies.rs`.
  - **List mode**: Show all homies in a scrollable `List`. Keybinds: `j/k` scroll, `Enter` select, `d` delete (with modal), `r` rename (inline form), `i` insert (inline form), `f` manage favorites.
  - **Add mode**: Inline form with `InputForm` widget for name. `Enter` submits, `Esc` cancels.
  - **Rename mode**: Inline form pre-filled with current name.
  - **Favorites mode**: Split-pane. Left: homie list. Right: `MultiSelect`-like list of all restaurants with checkboxes (`[x]`/`[ ]`). `Space` toggles. `s` saves, `Esc` cancels.
  - Use existing feature functions: `create_homie`, `delete_homie` (or equivalent), `get_all_homies`, `add_homies_favorite_restaurant`, `remove_homies_favorite_restaurant`, `get_homies_favorite_restaurants`.

  **Must NOT do**:
  - Do NOT change feature logic — call existing functions only.
  - Do NOT allow delete without confirmation modal.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Multiple modes (list, add, edit, favorites) with complex state transitions.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T6, T8-T14)
  - **Parallel Group**: Wave 2
  - **Blocks**: T16
  - **Blocked By**: T2, T4

  **References**:
  - `src/features/homies.rs` — All homie-related functions.
  - `src/features/homies_favorites.rs` — Favorite restaurant functions.
  - `src/interaction.rs` — Current dialoguer flow for favorites (copy logic, not code).

  **Acceptance Criteria**:
  - [ ] Homies list renders.
  - [ ] Can add a homie via inline form.
  - [ ] Can delete a homie with confirmation.
  - [ ] Can toggle favorite restaurants and save.

  **QA Scenarios**:
  ```
  Scenario: Add and delete homie
    Tool: interactive_bash (tmux)
    Preconditions: TUI launches
    Steps:
      1. Navigate to Homies (j/k + Enter).
      2. Press `i`, type "TestHomie", press Enter.
      3. Assert "TestHomie" appears in list.
      4. Select "TestHomie", press `d`, confirm with `y`.
      5. Assert "TestHomie" no longer in list.
    Expected Result: Homie added then removed.
    Evidence: .sisyphus/evidence/t7-homies-crud.log
  ```

  **Commit**: YES (groups with Wave 2)

- [ ] T8. **Restaurants view (list, add, delete, rename)**

  **What to do**:
  - Create `src/tui/views/restaurants.rs`.
  - **List mode**: Scrollable `List` of restaurants. `j/k` scroll, `i` insert, `d` delete (modal), `r` rename (inline form).
  - Use existing functions: `create_restaurant`, `get_all_restaurants`.
  - Note: `delete_restaurant` may not exist in features yet — if missing, implement it as a minimal feature addition (or note as blocked and create a stub).

  **Must NOT do**:
  - Do NOT implement complex restaurant detail view (keep it list + CRUD).

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Similar to Homies but simpler (no favorites sub-mode).
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T6, T7, T9-T14)
  - **Parallel Group**: Wave 2
  - **Blocks**: T16
  - **Blocked By**: T4

  **References**:
  - `src/features/restaurants.rs` — CRUD functions.

  **Acceptance Criteria**:
  - [ ] Restaurant list renders.
  - [ ] Can add and delete restaurants.

  **QA Scenarios**:
  ```
  Scenario: Add restaurant
    Tool: interactive_bash (tmux)
    Preconditions: TUI launches
    Steps:
      1. Navigate to Restaurants.
      2. Press `i`, type "Testaurant", press Enter.
      3. Assert "Testaurant" appears in list.
    Expected Result: Restaurant added successfully.
    Evidence: .sisyphus/evidence/t8-restaurant-add.log
  ```

  **Commit**: YES (groups with Wave 2)

- [ ] T9. **Recipes view (list, add, delete, view details, import URL)**

  **What to do**:
  - Create `src/tui/views/recipes.rs`.
  - **List mode**: Scrollable `List` of recipe names. `j/k` scroll, `Enter` view details, `i` add (form with name, desc, prep, cook, servings), `d` delete (modal), `u` import from URL (inline form for URL).
  - **Detail mode**: Show recipe name, description, prep/cook time, servings, steps, ingredients. `Esc` returns to list.
  - Use existing functions: `create_recipe`, `delete_recipe`, `get_recipe_by_name`, `list_recipes`, `import_recipe_from_url`.

  **Must NOT do**:
  - Do NOT implement recipe editing (steps/ingredients) in Wave 2 — keep to add/delete/view/import.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Multiple modes and detail view with structured data.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T6-T8, T10-T14)
  - **Parallel Group**: Wave 2
  - **Blocks**: T16
  - **Blocked By**: T2, T4

  **References**:
  - `src/features/recipes.rs` — Recipe CRUD and import.

  **Acceptance Criteria**:
  - [ ] Recipe list renders.
  - [ ] Can view recipe details.
  - [ ] Can import recipe from URL.

  **QA Scenarios**:
  ```
  Scenario: View recipe details
    Tool: interactive_bash (tmux)
    Preconditions: TUI launches, at least one recipe exists
    Steps:
      1. Navigate to Recipes.
      2. Select a recipe, press Enter.
      3. Assert detail view shows recipe name and description.
    Expected Result: Detail view renders correctly.
    Evidence: .sisyphus/evidence/t9-recipe-detail.log
  ```

  **Commit**: YES (groups with Wave 2)

- [ ] T10. **Meal Plan view (list, add, remove, view range, generate shopping list)**

  **What to do**:
  - Create `src/tui/views/meal_plan.rs`.
  - **List mode**: Show meal plan entries for selected date range (default next 7 days). Columns: Date, Slot, Recipe Name.
  - **Add mode**: Inline form with recipe name (autocomplete or list selection), date (`YYYY-MM-DD`), slot (Breakfast/Lunch/Dinner/Snack dropdown using `List`).
  - **Remove**: Select entry, `d` to delete with confirmation.
  - **Shopping List**: `s` key generates shopping list for displayed date range and switches to Shopping List view (or shows in modal).
  - Use existing functions: `create_meal_plan_entry`, `delete_meal_plan_entry`, `list_meal_plan_entries`, `generate_shopping_list`.

  **Must NOT do**:
  - Do NOT implement calendar widget — use text input for dates.
  - Do NOT implement drag-and-drop reordering.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Date handling, multi-step add flow, shopping list integration.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T6-T9, T11-T14)
  - **Parallel Group**: Wave 2
  - **Blocks**: T16
  - **Blocked By**: T2, T4

  **References**:
  - `src/features/meal_plan_entries.rs` — Entry CRUD.
  - `src/features/shopping_list.rs` — Shopping list generation.

  **Acceptance Criteria**:
  - [ ] Meal plan list renders for date range.
  - [ ] Can add entry with recipe, date, slot.
  - [ ] Can generate shopping list.

  **QA Scenarios**:
  ```
  Scenario: Add meal plan entry
    Tool: interactive_bash (tmux)
    Preconditions: TUI launches, at least one recipe exists
    Steps:
      1. Navigate to Meal Plan.
      2. Press `i`, select recipe, enter date, select slot.
      3. Assert entry appears in list.
    Expected Result: Entry added.
    Evidence: .sisyphus/evidence/t10-mealplan-add.log
  ```

  **Commit**: YES (groups with Wave 2)

- [ ] T11. **Pantry view (list, add, update, remove)**

  **What to do**:
  - Create `src/tui/views/pantry.rs`.
  - **List mode**: Scrollable `Table` with columns: Ingredient, Quantity, Measure.
  - **Add mode**: Inline form with ingredient name, quantity, measure.
  - **Update mode**: Select item, `e` to edit quantity/measure.
  - **Remove**: Select item, `d` to delete.
  - Use existing functions: `add_pantry_ingredient`, `update_pantry_ingredient`, `remove_pantry_ingredient`, `list_pantry_ingredients`.

  **Must NOT do**:
  - Do NOT implement unit conversion — just store raw values.

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple CRUD table.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T6-T10, T12-T14)
  - **Parallel Group**: Wave 2
  - **Blocks**: T16
  - **Blocked By**: T4

  **References**:
  - `src/features/pantry_ingredients.rs` — Pantry CRUD.

  **Acceptance Criteria**:
  - [ ] Pantry list renders as table.
  - [ ] Can add, update, remove items.

  **QA Scenarios**:
  ```
  Scenario: Update pantry item
    Tool: interactive_bash (tmux)
    Preconditions: TUI launches, pantry has items
    Steps:
      1. Navigate to Pantry.
      2. Select item, press `e`, change quantity.
      3. Assert updated quantity displays.
    Expected Result: Quantity updated.
    Evidence: .sisyphus/evidence/t11-pantry-update.log
  ```

  **Commit**: YES (groups with Wave 2)

- [ ] T12. **Shopping List view (display generated list)**

  **What to do**:
  - Create `src/tui/views/shopping_list.rs`.
  - Display a read-only `Table` of shopping list items: Ingredient, Quantity to Buy, Measure.
  - Data is populated by navigating from Meal Plan view (`s` key) or by selecting a date range in this view.
  - Minimal view — primarily a display target.

  **Must NOT do**:
  - Do NOT implement shopping list persistence — it's always generated on demand.

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Read-only table display.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T6-T11, T13-T14)
  - **Parallel Group**: Wave 2
  - **Blocks**: T16
  - **Blocked By**: T4

  **References**:
  - `src/features/shopping_list.rs` — `generate_shopping_list`.

  **Acceptance Criteria**:
  - [ ] Shopping list renders as table when data available.
  - [ ] Shows placeholder text when empty.

  **QA Scenarios**:
  ```
  Scenario: Display shopping list
    Tool: Bash (cargo test)
    Preconditions: T1-T11 complete
    Steps:
      1. Render ShoppingList view with mock data.
    Expected Result: Table renders correctly.
    Evidence: .sisyphus/evidence/t12-shopping-list-test.log
  ```

  **Commit**: YES (groups with Wave 2)

- [ ] T13. **Templates view (list, add, remove, apply)**

  **What to do**:
  - Create `src/tui/views/templates.rs`.
  - **List mode**: Table with columns: Name, Day, Slot, Recipe.
  - **Add mode**: Inline form with name, day (0-6), slot (dropdown), recipe (list select).
  - **Remove**: Select, `d` to delete.
  - **Apply**: `a` key applies templates to a date range (prompt for start date and days).
  - Use existing functions: `create_meal_plan_template`, `delete_meal_plan_template`, `list_meal_plan_templates`, `apply_meal_plan_templates`.

  **Must NOT do**:
  - Do NOT implement template editing — remove and re-add instead.

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple CRUD + apply action.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T6-T12, T14)
  - **Parallel Group**: Wave 2
  - **Blocks**: T16
  - **Blocked By**: T4

  **References**:
  - `src/features/meal_plan_templates.rs` — Template functions.

  **Acceptance Criteria**:
  - [ ] Template list renders.
  - [ ] Can apply templates to date range.

  **QA Scenarios**:
  ```
  Scenario: Apply templates
    Tool: interactive_bash (tmux)
    Preconditions: TUI launches, templates exist
    Steps:
      1. Navigate to Templates.
      2. Press `a`, enter start date and days.
      3. Assert success notification.
    Expected Result: Templates applied.
    Evidence: .sisyphus/evidence/t13-templates-apply.log
  ```

  **Commit**: YES (groups with Wave 2)

- [ ] T14. **Pick flow view (select homies → pick restaurant → show result)**

  **What to do**:
  - Create `src/tui/views/pick.rs`.
  - **Step 1 (Homies)**: Multi-select list of all homies (`Space` to toggle, `a` to select all). `Enter` proceeds.
  - **Step 2 (Picking)**: Shows "Picking..." spinner (or just a message) while `get_candidate_restaurants` runs async.
  - **Step 3 (Result)**: Displays selected restaurant name. `Enter` or `r` to re-pick (reroll), `s` to save (add to recent), `q` or `Esc` to go back.
  - Use existing functions: `get_all_homies`, `get_candidate_restaurants`, `add_recent_restaurant_for_homies`.
  - Replicate the logic from `AppState::work()` in `src/main.rs:83-118`.

  **Must NOT do**:
  - Do NOT implement food item picking (out of scope — the CLI `Pick` only picks restaurants currently).
  - Do NOT change the candidate logic — use existing functions exactly.

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Multi-step wizard flow with async loading and state machine.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T6-T13)
  - **Parallel Group**: Wave 2
  - **Blocks**: T16
  - **Blocked By**: T2, T4

  **References**:
  - `src/main.rs:83-118` — Existing `AppState::work()` pick flow.
  - `src/features/recents.rs` — Recent restaurant functions.
  - `src/features/homies.rs` — `get_all_homies`.
  - `src/features/homies_favorites.rs` — `get_candidate_restaurants`.

  **Acceptance Criteria**:
  - [ ] Can select homies and proceed.
  - [ ] Picking loads candidates asynchronously.
  - [ ] Result displays restaurant name.
  - [ ] Can re-pick or save.

  **QA Scenarios**:
  ```
  Scenario: Full pick flow
    Tool: interactive_bash (tmux)
    Preconditions: TUI launches, homies and restaurants exist
    Steps:
      1. Navigate to Pick.
      2. Select homies, press Enter.
      3. Wait for result screen.
      4. Assert a restaurant name is displayed.
      5. Press `r` to reroll, assert different (or same valid) restaurant.
    Expected Result: Pick flow completes end-to-end.
    Evidence: .sisyphus/evidence/t14-pick-flow.log
  ```

  **Commit**: YES (groups with Wave 2)

- [ ] T15. **TUI state/logic unit tests**

  **What to do**:
  - Add tests in `src/tui/app.rs` (or `src/tui/tests.rs`):
    - `app_quit_on_q` — pressing `q` sets `should_quit`.
    - `app_navigate_sidebar` — `j` increments `sidebar_selected`, `k` decrements (with bounds).
    - `app_switch_view` — `Enter` on sidebar switches `current_view`.
    - `app_notification_lifecycle` — error sets notification, next tick clears it (or explicit dismiss).
  - Add tests for widgets in `src/tui/widgets/`:
    - `input_form_typing` — chars append, backspace removes.
    - `modal_selection` — arrow keys change selected option.
  - Use `ratatui::backend::TestBackend` for rendering tests where applicable.

  **Must NOT do**:
  - Do NOT test DB interactions here — mock the state.
  - Do NOT test crossterm event reading — test the `App::on_event` handler with synthetic `AppEvent`s.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Requires careful test design for state machines.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T6-T14)
  - **Parallel Group**: Wave 2
  - **Blocks**: T17
  - **Blocked By**: T7, T9, T10, T14 (tests for state logic introduced in those views)

  **References**:
  - `src/tui/app.rs` — State to test.
  - `src/tui/event.rs` — `AppEvent` enum to use as test inputs.
  - Ratatui `TestBackend` docs.

  **Acceptance Criteria**:
  - [ ] `cargo test --lib tui` passes with all new tests.
  - [ ] Test coverage for `App` state transitions ≥ 80% of event variants.

  **QA Scenarios**:
  ```
  Scenario: State transition tests pass
    Tool: Bash (cargo test)
    Preconditions: T1-T14 complete
    Steps:
      1. Run `cargo test --lib tui`
    Expected Result: All tests pass, no failures.
    Evidence: .sisyphus/evidence/t15-unit-tests.log
  ```

  **Commit**: YES (separate commit for tests)
  - Message: `test(tui): add unit tests for app state and widget logic`

- [ ] T16. **Wire all views into sidebar navigation + keybindings**

  **What to do**:
  - In `src/tui/app.rs`, implement `on_event` dispatch:
    - Global keys: `q` → quit (if not in insert mode), `Esc` → back/cancel, `:` → command mode (optional, for search), `/` → search within current view.
    - Sidebar keys: `h` / `←` focus sidebar, `l` / `→` focus main, `j` / `↓` next item, `k` / `↑` prev item, `g` top, `G` bottom, `Enter` select view.
    - Delegate view-specific events to the active view module (e.g., `views::homies::on_event(&mut self.homies_state, event)`).
  - In `src/tui/ui.rs`, render the active view in the main content area by matching `app.current_view`.
  - Ensure `Command::Tui` in `main.rs` properly initializes the `App` and enters the event loop.

  **Must NOT do**:
  - Do NOT change existing `main.rs` CLI dispatch beyond adding the `Tui` arm.
  - Do NOT remove `dialoguer`-based interactive flows from `src/interaction.rs` (they stay for CLI compatibility).

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Wiring and dispatch logic.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (must come after all views)
  - **Parallel Group**: Wave 3 (Sequential)
  - **Blocks**: T17
  - **Blocked By**: T5, T6, T7, T8, T9, T10, T11, T12, T13, T14

  **References**:
  - `src/tui/app.rs` — State and event handler.
  - `src/tui/ui.rs` — Render dispatcher.
  - `src/main.rs` — Entry point.

  **Acceptance Criteria**:
  - [ ] `cargo run -- tui` launches and all sidebar items are reachable.
  - [ ] `j/k` or `↓/↑` navigate sidebar.
  - [ ] `Enter` opens selected view.
  - [ ] `q` quits from any view.
  - [ ] `Esc` cancels insert/modal mode.

  **QA Scenarios**:
  ```
  Scenario: Navigate all sidebar items
    Tool: interactive_bash (tmux)
    Preconditions: TUI launches
    Steps:
      1. Launch TUI.
      2. Press `j` repeatedly to cycle through sidebar.
      3. Press `Enter` on each item.
      4. Assert main content changes for each view.
      5. Press `q` to quit.
    Expected Result: All views accessible, quit works.
    Evidence: .sisyphus/evidence/t16-navigation.log
  ```

  **Commit**: YES (groups with Wave 3)
  - Message: `feat(tui): wire sidebar navigation and keybindings to all views`

- [ ] T17. **End-to-end QA: launch TUI, navigate all views, perform CRUD**

  **What to do**:
  - Run `cargo run -- tui` in a tmux session.
  - Systematically navigate to each view and perform a basic action:
    - Dashboard: Verify counts load.
    - Homies: Add "QAHomie", verify in list, delete.
    - Restaurants: Add "QARestaurant", verify, delete.
    - Recipes: View list.
    - Meal Plan: View list.
    - Pantry: Add "Flour", qty 1, measure "cup", verify.
    - Shopping List: View (may be empty).
    - Templates: View list.
    - Pick: Select homies, run pick, view result.
  - Capture terminal output (`tmux capture-pane`) or use `script` for session recording.
  - Save evidence to `.sisyphus/evidence/final-qa/`.

  **Must NOT do**:
  - Do NOT skip views — every sidebar item must be visited.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Requires systematic manual verification via tmux.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on all views)
  - **Parallel Group**: Wave 3
  - **Blocks**: T18, T19
  - **Blocked By**: T15, T16

  **Acceptance Criteria**:
  - [ ] All 9 sidebar sections visited.
  - [ ] CRUD operations succeed in Homies, Restaurants, Pantry.
  - [ ] Pick flow completes.
  - [ ] No panics or terminal corruption.

  **QA Scenarios**:
  ```
  Scenario: Full end-to-end walkthrough
    Tool: interactive_bash (tmux)
    Preconditions: Fresh DB or known state
    Steps:
      1. Start tmux, run `cargo run -- tui`.
      2. Visit each sidebar item, perform basic action.
      3. Capture pane output.
    Expected Result: All actions succeed, terminal clean after quit.
    Evidence: .sisyphus/evidence/final-qa/e2e-walkthrough.log
  ```

  **Commit**: NO (QA is verification, not code change)

- [ ] T18. **Final code quality review (clippy, formatting)**

  **What to do**:
  - Run `cargo clippy --all-targets --all-features` and fix all warnings.
  - Run `cargo fmt --check` and fix if needed.
  - Run `cargo test` and ensure all tests pass.
  - Remove any `println!` or `dbg!` left in TUI code.
  - Check for unused imports in `src/tui/`.

  **Must NOT do**:
  - Do NOT fix clippy warnings in existing non-TUI code unless trivial (avoid scope creep).

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Automated checks and minor fixes.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T19)
  - **Parallel Group**: Wave 3
  - **Blocks**: —
  - **Blocked By**: T17

  **Acceptance Criteria**:
  - [ ] `cargo clippy` passes (or has only pre-existing warnings documented).
  - [ ] `cargo fmt --check` passes.
  - [ ] `cargo test` passes.

  **QA Scenarios**:
  ```
  Scenario: Quality checks pass
    Tool: Bash
    Preconditions: T17 complete
    Steps:
      1. Run `cargo clippy --all-targets --all-features`.
      2. Run `cargo fmt --check`.
      3. Run `cargo test`.
    Expected Result: All pass.
    Evidence: .sisyphus/evidence/t18-quality.log
  ```

  **Commit**: YES (if fixes made)
  - Message: `style(tui): fix clippy warnings and format`

- [ ] T19. **Update README with TUI usage instructions**

  **What to do**:
  - Add a "TUI" section to `README.md`.
  - Document: `cargo run -- tui` or `lunch_picker tui`.
  - Keybindings reference table (Navigation, Actions, Global).
  - Screenshots or ASCII art of layout (optional).
  - Note that existing CLI commands remain available.

  **Must NOT do**:
  - Do NOT remove existing README content.

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Documentation writing.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T18)
  - **Parallel Group**: Wave 3
  - **Blocks**: —
  - **Blocked By**: T17

  **Acceptance Criteria**:
  - [ ] README includes TUI launch command.
  - [ ] README includes keybindings table.
  - [ ] README notes CLI compatibility.

  **QA Scenarios**:
  ```
  Scenario: README updated
    Tool: Bash
    Preconditions: T17 complete
    Steps:
      1. Read `README.md`.
      2. Assert "TUI" section exists with keybindings.
    Expected Result: Documentation present.
    Evidence: .sisyphus/evidence/t19-readme.log
  ```

  **Commit**: YES
  - Message: `docs(readme): add TUI usage and keybindings`

---

## Final Verification Wave

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in `.sisyphus/evidence/`. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy`, `cargo fmt --check`, `cargo test`. Review all changed files for: `unwrap()`/`expect()` in TUI code, unused imports, commented-out code. Check AI slop: excessive comments, over-abstraction, generic names.
  Output: `Clippy [PASS/FAIL] | Fmt [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state. Launch TUI via `cargo run -- tui`. Navigate every sidebar item. Verify data loads. Add a homie, add a restaurant, run Pick flow. Capture terminal screenshots or `script` output. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git log/diff). Verify 1:1 — everything in spec was built, nothing beyond spec was built. Check "Must NOT do" compliance. Detect cross-task contamination.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **T1-T5**: `feat(tui): scaffold ratatui module and event loop`
- **T6-T14**: `feat(tui): add views for homies, restaurants, recipes, plans, pantry, shopping, templates, pick`
- **T15**: `test(tui): add unit tests for state transitions`
- **T16-T19**: `feat(tui): integrate navigation, polish, and docs`
- **F1-F4 fixes**: `fix(tui): address review feedback`

---

## Success Criteria

### Verification Commands
```bash
cargo build --release  # Expected: zero errors
cargo clippy           # Expected: zero warnings (or allowed ones documented)
cargo test             # Expected: all tests pass
cargo run -- tui       # Expected: TUI launches, sidebar renders, q quits
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass
- [ ] TUI launches and all views are navigable
- [ ] Pick flow works end-to-end
- [ ] Existing CLI commands unaffected
