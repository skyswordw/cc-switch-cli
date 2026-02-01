# TUI Add Forms (Provider + MCP) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the current JSON-only “Add Provider” / “Add MCP Server” flow with a friendly, scooter-style form UI (templates + grouped fields + live JSON preview) while keeping an escape hatch to advanced JSON editing.

**Architecture:** Introduce a new `App::form` state machine (`ProviderAdd`, `McpAdd`) driven by key events. Render it in `cli/tui/ui.rs` with reusable mini-components (chip bar, field list, input box, JSON preview). On submit, generate JSON and reuse the existing `EditorSubmit::ProviderAdd` / `EditorSubmit::McpAdd` handling for persistence. Advanced JSON editing reuses `EditorState`; saving applies JSON back into the form (round-trip), without creating the entity until the user submits from the form.

**Tech Stack:** Rust, ratatui, crossterm, serde_json, toml

---

### Task 1: Add reusable form primitives

**Files:**
- Create: `src-tauri/src/cli/tui/form.rs`
- Modify: `src-tauri/src/cli/tui/mod.rs`
- Test: `src-tauri/src/cli/tui/app.rs`

**Step 1: Write failing tests for single-line text input editing**

Add unit tests covering: `insert`, `backspace`, `left/right/home/end`, and UTF-8 cursor safety.

Run: `cd src-tauri && cargo test text_input_ -- -q`
Expected: FAIL (types/functions not found).

**Step 2: Implement `TextInput` + helpers**

Implement `TextInput` with:
- `value: String`, `cursor: usize` (char index)
- editing ops: insert char, backspace/delete, move left/right/home/end, set value
- `display_value(mask: bool)` helper for secret-like fields

**Step 3: Run tests**

Run: `cd src-tauri && cargo test text_input_ -- -q`
Expected: PASS.

---

### Task 2: Add form state machine to `App`

**Files:**
- Modify: `src-tauri/src/cli/tui/app.rs`
- Test: `src-tauri/src/cli/tui/app.rs`

**Step 1: Write failing tests for “press `a` opens add-form”**

Add tests:
- Providers page: `a` opens `FormState::ProviderAdd` (not JSON editor)
- MCP page: `a` opens `FormState::McpAdd`

Run: `cd src-tauri && cargo test providers_a_opens_add_form mcp_a_opens_add_form -- -q`
Expected: FAIL.

**Step 2: Implement `App::form` wiring**

- Add `form: Option<FormState>` to `App`
- In `on_key`, handle form before global actions (`Esc`/`q` back)
- Add `on_form_key(...)` to process input + focus + submit/cancel

**Step 3: Run tests**

Run: `cd src-tauri && cargo test providers_a_opens_add_form mcp_a_opens_add_form -- -q`
Expected: PASS.

---

### Task 3: Provider add form (templates + fields + JSON)

**Files:**
- Modify: `src-tauri/src/cli/tui/app.rs`
- Modify: `src-tauri/src/cli/tui/ui.rs`
- Modify: `src-tauri/src/cli/i18n.rs`
- Test: `src-tauri/src/cli/tui/app.rs`

**Step 1: Write failing tests for provider JSON generation**

Add tests:
- `Ctrl+S` on ProviderAdd form returns `Action::EditorSubmit { submit: ProviderAdd, content }`
- JSON includes `id/name/settingsConfig` and uses `settingsConfig.env` keys for the active `AppType`

Run: `cd src-tauri && cargo test provider_add_form_ -- -q`
Expected: FAIL.

**Step 2: Implement ProviderAdd form model**

Model requirements:
- template chips (at least: `Custom`, `Official` per app)
- fields: `id`, `name`, `websiteUrl`, `notes`
- app-specific fields:
  - Claude: `ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_BASE_URL`, optional model envs
  - Codex: `base_url`, `model`, `wire_api`, `requires_openai_auth`, optional `env_key`, optional `OPENAI_API_KEY`
  - Gemini: `auth_type` (oauth/api-key), optional `GEMINI_API_KEY`, `GOOGLE_GEMINI_BASE_URL`, optional `GEMINI_MODEL`
- auto-generate `id` from `name` (unless user edited `id` manually)
- validation: id/name required (toast on missing)

**Step 3: Implement ProviderAdd form rendering**

In `ui.rs` render:
- top key bar (mode-aware)
- template chip bar (wrap-aware)
- left: field list + field editor (scooter-style input box)
- right: live JSON preview (scrollable)
- focused border uses theme accent

**Step 4: Run tests**

Run: `cd src-tauri && cargo test provider_add_form_ -- -q`
Expected: PASS.

---

### Task 4: MCP add form (fields + JSON)

**Files:**
- Modify: `src-tauri/src/cli/tui/app.rs`
- Modify: `src-tauri/src/cli/tui/ui.rs`
- Modify: `src-tauri/src/cli/i18n.rs`
- Test: `src-tauri/src/cli/tui/app.rs`

**Step 1: Write failing tests for MCP JSON generation**

Add tests:
- `Ctrl+S` on McpAdd form returns `EditorSubmit::McpAdd`
- JSON includes `id/name/server/apps` with `server.command` and `server.args`

Run: `cd src-tauri && cargo test mcp_add_form_ -- -q`
Expected: FAIL.

**Step 2: Implement McpAdd form model + rendering**

Fields:
- `id`, `name`
- `command`
- `args` (single-line; split by whitespace into JSON array for `server.args`)
- `apps` checkboxes (Claude/Codex/Gemini)

**Step 3: Run tests**

Run: `cd src-tauri && cargo test mcp_add_form_ -- -q`
Expected: PASS.

---

### Task 5: Advanced JSON round-trip + i18n polish

**Files:**
- Modify: `src-tauri/src/cli/tui/app.rs`
- Modify: `src-tauri/src/cli/i18n.rs`
- Test: `src-tauri/src/cli/tui/app.rs`

**Step 1: Write failing tests**

Add tests:
- From ProviderAdd/McpAdd form press `e` opens editor with `EditorSubmit::FormApplyJson { .. }`
- In that editor, `Ctrl+S` parses JSON and updates form fields, then closes editor
- Invalid JSON keeps editor open and shows error toast

Run: `cd src-tauri && cargo test form_apply_json_ -- -q`
Expected: FAIL.

**Step 2: Implement `FormApplyJson` editor submit and apply logic**

Implementation notes:
- parsing provider JSON can be via `serde_json::Value` → extract known keys (ignore unknown)
- Codex config parsing uses `toml::from_str::<toml::Table>()` to pull `base_url/model/wire_api/env_key/requires_openai_auth`

**Step 3: Add missing i18n strings**

Add texts for: templates title, field labels, focus names, “JSON preview”, “Advanced JSON”, validation toasts.

**Step 4: Run tests**

Run: `cd src-tauri && cargo test form_apply_json_ -- -q`
Expected: PASS.

---

### Task 6: Small correctness fix + verification

**Files:**
- Modify: `src-tauri/src/cli/tui/data.rs`
- Test: `src-tauri/src/cli/tui/data.rs`

**Step 1: Write failing test**

Add a unit test for Gemini `api_url` extraction to prefer `GOOGLE_GEMINI_BASE_URL`.

Run: `cd src-tauri && cargo test gemini_api_url_ -- -q`
Expected: FAIL.

**Step 2: Implement fix**

Update `extract_api_url()` to check:
- `GOOGLE_GEMINI_BASE_URL` (preferred)
- fallback to legacy keys if present

**Step 3: Run verification**

Run:
- `cd src-tauri && cargo fmt`
- `cd src-tauri && cargo test`
- `cd src-tauri && cargo clippy --all-targets -- -D warnings` (ok to document existing failures)

