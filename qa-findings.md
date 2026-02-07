# QA Findings – Elite Rust

Systematic testing of implemented features (01–06). Issues and fix suggestions; updated as fixes are applied.

---

## Feature 01: Core Gameplay

### Test cases
- [x] Commander name: empty rejected and re-prompts
- [x] help, status, quit work when given valid name first

### Findings
- **No critical bugs.** Empty name is correctly rejected. (Test script ordering: use a non-command as name when piping so first real command is help.)

---

## Feature 02: Star Systems & Travel

### Test cases
- [x] systems: lists reachable systems with distances
- [x] info: LAVE works, "lave" fails (case-sensitive)
- [x] jump: no args → usage; invalid/xyz → not found; "diso" → not found (case-sensitive); "DISO" + n → cancelled

### Findings
1. **System names case-sensitive** – `info lave` and `jump diso` fail; only uppercase (e.g. LAVE, DISO) works. **Fix:** Case-insensitive lookup (normalize input to match stored names, e.g. uppercase).
2. **UX:** Players see "DISO" in systems list but must type exact case; forgiving lookup improves playability.

---

## Feature 03: Trading

### Test cases
- [x] market: works
- [x] buy/sell: require both &lt;item&gt; and &lt;quantity&gt;

### Findings
1. **README mismatch** – README says `buy <commodity> [amount]` (amount optional). Code requires both item and quantity. **Fix:** Either default quantity to 1 when omitted, or update README to "buy &lt;item&gt; &lt;quantity&gt;".
2. **UX:** Supporting "buy food" (default 1) is friendlier.

---

## Feature 04: Ship Upgrades

### Test cases
- [x] upgrade: shows options and credits
- [x] ships: lists ships
- **Note:** Help says "upgrade buy &lt;item&gt;" but code expects "upgrade &lt;ship&gt;" (ship name). README/help could say "upgrade &lt;ship_name&gt;" for clarity.

### Findings
- **Doc/help only:** Align help text with actual usage (upgrade &lt;ship&gt;, not "upgrade buy &lt;item&gt;").

---

## Feature 05: Save/Load

### Test cases
- [x] save with no args → Usage
- [x] load with no args → Usage
- [x] saves: lists saves (slot names from metadata)

### Findings
1. **Bug – save/load never succeed with "save slot1"** – Handler requires `args.len() == 2` and uses `args[1]` for slot name. For "save slot1", main passes `args = ["slot1"]` (one element). So `args.len() != 2` and we always hit usage; also `args[1]` would be out of bounds. **Fix:** Require `args.len() >= 1` and use `args[0]` as slot_name.
2. **Same for load and delete** – They use `args[1]`; should use `args[0]`.
3. **README** says "save [name]" (optional); code requires slot. Either add default slot (e.g. "default") when omitted or keep required and update README.

---

## Feature 06: Dynamic Events

### Test cases
- [ ] Jump with events triggered (probabilistic)
- [ ] Event choices and outcomes

### Findings
- Not fully exercised in this run (events are random). Code path exists; manual play or repeated jumps needed to verify.

---

## Fix log (implemented)

1. **Save/Load/Delete args (Feature 05)** – Handlers required `args.len() == 2` and used `args[1]` for slot name. Main passes `["slot1"]` for "save slot1". Fixed: require `args.is_empty()` and use `args[0]` for slot name in `handle_save`, `handle_load`, `handle_delete_save` (`src/game/save_load.rs`).
2. **Case-insensitive system names (Feature 02)** – `info lave` and `jump diso` failed. Added `SystemManager::get_system_case_insensitive()` in `src/data/systems.rs`; travel and info use it in `travel.rs` and via `get_system_info`.
3. **Buy/Sell args and optional quantity (Feature 03)** – Handler expected 3 args and used `args[1]`/`args[2]`; main passes 2 args (`["food", "5"]`). Fixed: use `args[0]` for item, `args.get(1).unwrap_or("1")` for quantity; allow `buy <item>` (default quantity 1). Updated usage to "buy <item> [quantity]" and "sell <item> [quantity]".

**Optional polish (not done):** Travel confirmation could show canonical system name (e.g. "To: DISO") when user types "diso".
