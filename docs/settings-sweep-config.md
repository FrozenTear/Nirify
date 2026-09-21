# Nirify Config sweep — findings only

Audit of [FrozenTear/Nirify](https://github.com/FrozenTear/Nirify) `master` at `27e4bc0` (includes [#21](https://github.com/FrozenTear/Nirify/pull/21) import/`smart_replace` data-loss fix, [#22](https://github.com/FrozenTear/Nirify/pull/22) IPC layout snapshot, [#23](https://github.com/FrozenTear/Nirify/pull/23) Displays rearrange). **No merge. No feature work.**

**Assumed niri version:** latest **stable [v26.04](https://github.com/niri-wm/niri/releases/tag/v26.04)** (April 2026). Wiki pages cited below are from niri `docs/wiki/` at commit `dd75865` (same tree as the published wiki). Git-only options marked `Since: next release` are **out of scope** for “missing vs 26.04” except where Nirify already models them.

**Nirify version in tree:** 0.4.0.

**Primary niri docs:**

- [Configuration: Introduction](https://niri-wm.github.io/niri/Configuration%3A-Introduction.html) · [wiki](https://github.com/niri-wm/niri/wiki/Configuration:-Introduction)
- [Include](https://niri-wm.github.io/niri/Configuration%3A-Include.html) (25.11; `optional=true` and `~/` since 26.04)
- [Input](https://niri-wm.github.io/niri/Configuration%3A-Input.html)
- [Outputs](https://niri-wm.github.io/niri/Configuration%3A-Outputs.html)
- [Layout](https://niri-wm.github.io/niri/Configuration%3A-Layout.html)
- [Key Bindings](https://niri-wm.github.io/niri/Configuration%3A-Key-Bindings.html)
- [Window Rules](https://niri-wm.github.io/niri/Configuration%3A-Window-Rules.html)
- [Layer Rules](https://niri-wm.github.io/niri/Configuration%3A-Layer-Rules.html)
- [Animations](https://niri-wm.github.io/niri/Configuration%3A-Animations.html)
- [Gestures](https://niri-wm.github.io/niri/Configuration%3A-Gestures.html)
- [Miscellaneous](https://niri-wm.github.io/niri/Configuration%3A-Miscellaneous.html) (spawn, cursor, overview, clipboard, xwayland-satellite, hotkey-overlay, config-notification, blur)
- [Named Workspaces](https://niri-wm.github.io/niri/Configuration%3A-Named-Workspaces.html)
- [Switch Events](https://niri-wm.github.io/niri/Configuration%3A-Switch-Events.html)
- [Recent Windows](https://niri-wm.github.io/niri/Configuration%3A-Recent-Windows.html) (**since 25.11**, not 25.05)
- [Debug Options](https://niri-wm.github.io/niri/Configuration%3A-Debug-Options.html)
- [IPC](https://niri-wm.github.io/niri/IPC.html)

---

## A) Severity-ranked board

### Critical

- **`scale 1.0` is omitted on write, which means “auto-guess”, not “1.0”.** `generate_outputs_kdl` (`src/config/storage/display.rs`) skips `scale` when `|scale - 1.0| <= 0.001`. Niri’s default when scale is unset is to **guess** from physical size/resolution ([Outputs § scale](https://niri-wm.github.io/niri/Configuration%3A-Outputs.html#scale)), not 1.0. After #22, `apply_live_outputs_to_settings` copies live `logical.scale` into `OutputConfig.scale` (often `1.0`). Next save can drop an explicit 1× and let niri HiDPI-guess 2×. Model default is `scale: 1.0` (`OutputConfig::default`), so “never set” and “explicit 1.0” are indistinguishable. **Not a tiny fix** — needs `Option<f64>` (or an explicit-set flag) through model/loader/writer/snapshot/UI.

- **Catch-all `window-rule` (no `match`) is discarded on import except `geometry-corner-radius`.** `import_window_rules_from_doc` (`src/config/loader/import.rs`) skips rules without `match`; `import_appearance_from_doc` only lifts the first catch-all’s corner radius into `appearance.corner_radius`. A user catch-all with `opacity`, `clip-to-geometry`, `open-maximized`, `draw-border-with-background`, etc. is stripped by `smart_replace` and never lands in `window-rules.kdl`. `generate_appearance_kdl` then emits a **new** catch-all that only has `geometry-corner-radius`. First-run (#21) and launch absorb both go through this importer. **Not tiny** — identity/UI for “global rule vs Appearance corner radius” must be designed.

### High

- **Omit-default writers cannot override an earlier user `include` (last-wins footgun).** Nirify places `include "nirify/main.kdl"` last so its nodes win ([Include § Positionality](https://niri-wm.github.io/niri/Configuration%3A-Include.html#positionality)). Merging only changes properties that are written. Omitted defaults therefore **lose** to an earlier include. Concrete omits:
  - `center-focused-column "never"` (`generate_appearance_kdl`)
  - `default-column-display "normal"` (`generate_layout_extras_kdl`; window-rule writer only emits `"tabbed"`)
  - `scale` ≈ 1.0 (above)
  - `warp-mouse-to-focus` Off, `focus-follows-mouse` false, `workspace-auto-back-and-forth` false (`generate_behavior_kdl`)
  - `variable-refresh-rate` Off
  - #21 already warns on `ConflictingInclude` but does **not** import those files.

- **Binds last-wins vs Nirify first-wins.** Niri: later `binds` override the same key ([Include § Binds](https://niri-wm.github.io/niri/Configuration%3A-Include.html#binds)). Import: `doc.get("binds")` (first block) then included files **append**; `generate_keybindings_kdl` **skips** duplicate combos and keeps the first. First-run can keep the wrong action.

- **Include traversal order on import is not positional.** `import_from_niri_config_recursive_tracked` parses the **entire** current file, then walks includes. Niri applies includes **in document order** (content after an include overrides that include). A file `include "a.kdl"` then `layout { gaps 16 }` imports as “a.kdl wins”. Same inversion in `load_keybindings`.

- **Import jail vs niri 26.04 `~/` includes.** Import `resolve_include_path` expands `~/` then requires the canonical path under `$XDG_CONFIG_HOME/niri`. Niri 26.04 expands `~/` to the home dir with **no** such jail. `~/dotfiles/niri-binds.kdl` is skipped (warning only) and then stripped if it was inlined as managed nodes… it isn’t inlined; the include is preserved as unmanaged, but **first-run never copies those settings into `nirify/`**. `smart_replace` conflict scan **refuses** `~/` (`resolve_include_path` returns `None`), so those conflicts are invisible.

- **#22 snapshot clobbers `variable-refresh-rate on-demand=true`.** `update_output_from_live` sets `VrrMode::On`/`Off` from `vrr_enabled`. On-demand cannot be observed over IPC (documented in `outputs_layout.rs`). Import-connected-layout / rearrange seed will persist On/Off.

- **Output identity is connector-name only.** Niri matches `output` by connector **or** `"Make Model Serial"` ([Outputs](https://niri-wm.github.io/niri/Configuration%3A-Outputs.html)). #22/#23 match `FullOutputInfo.name` == `OutputConfig.name`. A make/model/serial block will not merge with the live connector row; absorb uses the same `name` key.

- **Fractional layout values rounded to integers.** `generate_appearance_kdl` does `gaps.round() as i32` and `field_f32_as_int` for focus-ring/border width and struts. Niri accepts fractional gaps/widths since 0.1.7 ([Layout § gaps](https://niri-wm.github.io/niri/Configuration%3A-Layout.html#gaps)). Import of `gaps 0.5` becomes `0` or `1` on next save.

- **Window-rule / tab-indicator / focus-ring / border gradients on rules are not modeled.** Loader `warn_gradients` (`src/config/loader/rules.rs`) states they “will be dropped if this rule is re-saved”. Global layout gradients **do** round-trip via `ColorOrGradient`.

### Medium

- **`optional=true` includes (26.04) are not first-class.** Preserved as unmanaged `include` nodes (good). No parse of the flag, no optional Nirify include, no UI. Missing-file optionals still reload in niri; Nirify does not create them.

- **Launch absorb (`absorb_stripped_nodes`) uses `FeatureCompat::all_enabled()`** because version IPC is async (`App::new`). Adopted `blur.kdl` / `recent-windows.kdl` can be written on old niri; `main.kdl` is not rewritten if it already exists (commented). First launch after #21 on 25.11 can leave a blur file that is not included — OK — but a later `save_settings` with detected compat still version-gates `main.kdl`. Residual: absorb of version-gated **nodes in `config.kdl`** still writes the category file.

- **Absorb identity is shallow.** Window/layer rules: `matches` + `excludes` only (same match, different opacity → not adopted). Outputs/workspaces: `name` only (make/model/serial vs connector). Keybindings: normalized combo (correct vs niri “one bind per key”, but first-wins). Catch-alls never enter the collection (Critical above).

- **`ConfigFile::HEALTH_CHECK` omits trackpoint, trackball, tablet, touch, keybindings, preferences.** Corrupt `input/trackpoint.kdl` etc. are not repaired by `repair_corrupted_configs`. Keybindings comment still says “loaded from niri config” — false since managed `keybindings.kdl`.

- **`ConfigPaths` has no `blur_kdl` field.** Load/save use `path_for(ConfigFile::Blur)` (correct). `Default` / takeover test fixtures omit it. Easy to miss in new path-based code.

- **Dead / leftover include helpers.** `ConfigPaths::add_include_line` appends include **without** last-wins or strip (unused). `migrate_include_line` is still called from `App::new` and rewrites `~/.config/.../nirify/main.kdl` → relative (still needed for pre-25.11 Nirify installs). Migration backups are named `config.kdl.backup.migration.*` — **not** listed by `list_backups` (looks for `config.kdl.backup-`, `.backup-`, or `.bak`).

- **`on-xdg-activate` (window-rule) is `Since: next release` — not in 26.04.** Completely absent from models/loader/writer. Fine for stable; will be a silent drop the day it ships if someone runs git niri.

- **`max-bpc` (output) and `debug { disable-10bit-output }` are also `next release`.** Not modeled.

- **`map-to-focused-window` (tablet, next release)** is loaded and **slashdashed** on write — good preserve pattern; do not emit live until niri releases it.

- **IPC surface is snapshot-oriented, not event-stream.** After #22: `get_full_outputs`, logical geometry, VRR supported/enabled. Still missing vs [IPC](https://niri-wm.github.io/niri/IPC.html): event stream, layers, pick-window, keyboard layouts, overview state, most `niri msg action` writes (Displays rearrange writes KDL, not `OutputConfig` IPC).

- **Appearance `window-rule` for corner radius lives in `appearance.kdl`, not `window-rules.kdl`.** Two files emit `window-rule`. Include order (`ConfigFile::ALL`) puts appearance before window-rules, so later rules can override — OK — but health/absorb/import treat them as different categories.

- **Recent-windows model comment says “v25.05+”; wiki and `NiriFeature::RecentWindows` correctly require 25.11.**

---

## B) Coverage matrix summary

Counted against **niri 26.04 stable** top-level / important nested sections (not every leaf). Nested “next release” items are listed but **not** counted as Missing-for-26.04.

| Status | Count | Items |
| --- | ---: | --- |
| **Present** | 18 | `input` (devices + mod-key / FFM / warp / power / track-layout / xkb file), `cursor`, `overview`, `gestures`, `clipboard`, `xwayland-satellite`, `blur`, `spawn-at-startup`, `spawn-sh-at-startup`, `environment`, `screenshot-path`, `prefer-no-csd`, `hotkey-overlay`, `config-notification`, `switch-events`, `recent-windows`, `animations`, `workspace` (named + layout override) |
| **Partial** | 8 | `layout`, `output`, `binds`, `window-rule`, `layer-rule` (gradients), `debug`, `include`, IPC |
| **Missing (26.04 top-level)** | 0 | — |
| **Missing (git / next)** | 3 | `on-xdg-activate`, output `max-bpc`, `debug.disable-10bit-output` |

### Per-section (Present / Partial / Missing)

| Area | Status | Notes |
| --- | --- | --- |
| `layout` | **Partial** | Gaps/struts/focus-ring/border/shadow/tab-indicator/insert-hint/presets/default-column-width/background-color/center-* modeled. Fractional gaps/widths coerced to int. `default-column-display "normal"` not written. Split across `appearance.kdl` + `advanced/layout-extras.kdl` + behavior fields. |
| `input` | **Present** | All device types + calibration + scroll-factor h/v + tablet `map-to-focused-output` (26.04, compat-gated / slashdashed). `map-to-focused-window` preserved slashdashed (not 26.04). |
| `output` | **Partial** | mode/custom/modeline/position/transform/VRR/focus-at-startup/backdrop/hot-corners/layout override. Scale 1.0 omit. No `max-bpc` (next). Identity = connector string. #22/#23 snapshot/rearrange. Deprecated top-level `background-color` still written (niri still accepts). |
| `binds` | **Partial** | Lossless `ActionNode`, 135-action catalog, overlay title / allow-when-locked / inhibit / cooldown / repeat. Duplicate combo = first wins (niri last). Scroll/mouse click combos stored as key names. |
| `window-rule` | **Partial** | Matches/excludes + opening + size + border/focus-ring/shadow/tab-indicator colors + 26.04 `background-effect` / `popups` + slashdash disabled rules. No catch-all import. No rule gradients. No `on-xdg-activate` (next). `default-column-display` only emits tabbed. |
| `layer-rule` | **Partial** | namespace / at-startup / layer (26.04) + opacity/shadow/radius/place-within-backdrop/baba-is-float/background-effect/popups. Gradients not modeled. |
| `workspace` | **Present** | name, `open-on-output`, layout override. |
| `animations` | **Present** | off/slowdown + all 11 named animations including `recent-windows-close`; spring/easing/cubic-bezier/custom shader. |
| `cursor` | **Present** | theme/size/hide-when-typing/hide-after-inactive-ms. |
| `overview` | **Present** | zoom, backdrop-color, workspace-shadow. |
| `gestures` | **Present** | dnd-edge-view-scroll, dnd-edge-workspace-switch, hot-corners (incl. per-output). |
| `clipboard` | **Present** | `disable-primary`. |
| `xwayland-satellite` | **Present** | default / off / custom path. |
| `blur` | **Present** | 26.04 top-level; file version-gated in `main.kdl`. |
| `spawn-at-startup` / `spawn-sh-at-startup` | **Present** | Separate files (`startup.kdl` / `misc.kdl`). |
| `environment` | **Present** | set + `null` unset. |
| `screenshot-path` | **Present** | default / null / custom. |
| `prefer-no-csd` | **Present** | |
| `hotkey-overlay` | **Present** | skip-at-startup, hide-not-bound. |
| `config-notification` | **Present** | disable-failed. |
| `debug` | **Partial** | All 26.04 flags in `DebugSettings` / `generate_debug_kdl`. Missing git-only `disable-10bit-output`. |
| `switch-events` | **Present** | lid/tablet spawn only (niri only documents `spawn`). |
| `recent-windows` | **Present** | off, delays, highlight, previews, binds (filter/scope/cooldown). Gated 25.11. |
| `include` | **Partial** | Relative `nirify/main.kdl` last; other includes preserved; conflict scan; no `optional=true`; no `~/` conflict scan; no absorb of conflicting includes. |
| IPC | **Partial** | version, windows, workspaces, focused window/output, full outputs (#22), reload, validate, quit. No event stream / layers / pick-window / actions. |

**Worst gaps (user-visible):** scale 1.0 → auto; catch-all window-rule import; omit-default vs earlier includes; binds first-wins; include path/`~/` jail; #22 VRR on-demand; output make/model/serial identity.

---

## C) KDL fidelity issues (post-#21)

#21 is **real and correctly ordered**: wizard `first_run_setup` = import → `smart_replace` → `save_settings`; launch `absorb_stripped_nodes` = import stripped nodes → merge (adopt-if-not-represented) → `save_dirty` **before** strip → `smart_replace`. Re-entry after `nirify/appearance.kdl` exists does not re-import a stripped `config.kdl`. Tests in `takeover.rs` cover the advertised cases.

### Include layout — good

- Writers emit `include "nirify/main.kdl"` (relative). `smart_replace` / `needs_rewrite` enforce **last** top-level node, dedupe Nirify includes, backup + parse-gate generated KDL (`src/config/replace.rs`).
- `generate_main_kdl` includes every `ConfigFile` except Preferences; RecentWindows / Blur skipped when `FeatureCompat` says so.
- niri 25.11 does not expand `~` in includes; Nirify still migrates the old tilde include. niri **26.04 does** expand `~/` — conflict scan and import jail are now **behind** niri.

### `MANAGED_NODES` vs writers — complete for 26.04

`MANAGED_NODES` (`replace.rs`) includes everything Nirify writes: `layout`, `input`, `animations`, `cursor`, `overview`, `output`, `workspace`, `window-rule`, `layer-rule`, `spawn-at-startup`, `spawn-sh-at-startup`, `environment`, `debug`, `switch-events`, `hotkey-overlay`, `screenshot-path`, `prefer-no-csd`, `focus-follows-mouse`, `warp-mouse-to-focus`, `workspace-auto-back-and-forth`, `binds`, `gestures`, `clipboard`, `xwayland-satellite`, `blur`, `config-notification`, `recent-windows`.

`focus-follows-mouse` / `warp-mouse-to-focus` / `workspace-auto-back-and-forth` are **not** valid niri top-level nodes (they live under `input {}`). Harmless extra strip names. `include` is correctly unmanaged.

### Node identity / round-trip

| Topic | Status |
| --- | --- |
| Disabled window/layer rules | Slashdash (`/-window-rule`) — Option 2, good |
| Version-gated tablet flags | Slashdash preserve |
| Keybind actions | Lossless `ActionNode` + raw Custom |
| Env unset | `null` |
| Empty `default-column-width {}` | Modeled (`ColumnWidthType::Auto` / `default_column_width_auto`) |
| Catch-all window-rule | **Broken** on import (Critical) |
| Rule gradients | **Dropped** on resave (High) |
| Scale / default-column-display / center-focused-column defaults | **Silent omit** (Critical/High) |
| Unmanaged nodes | Structure kept; **whitespace/comments normalized** by KDL Display (documented in `replace.rs`) |

### Silent drops (concrete)

| What | Where | Effect |
| --- | --- | --- |
| `scale 1.0` | `storage/display.rs` ~602 | niri auto-scale |
| `default-column-display "normal"` | `storage/layout_extras.rs`, `storage/rules.rs` | cannot override earlier `"tabbed"` |
| `center-focused-column "never"` | `storage/appearance.rs` | cannot override earlier always/on-overflow |
| Fractional gaps/widths/struts | `storage/appearance.rs` | 0.5 → 0/1 |
| Catch-all window-rule (non-radius) | `loader/import.rs` | deleted on takeover |
| Rule `*-gradient` | `loader/rules.rs` `warn_gradients` | dropped on save |
| Duplicate key combo | `storage/keybindings.rs` | later bind dropped |
| Malformed switch-event spawn | `storage/system.rs` | skipped + log |
| Invalid env name | `storage/system.rs` | skipped + log |
| `scroll-factor` 1.0 uniform | `storage/helpers.rs` | omit (OK — niri default) |
| VRR on-demand | `outputs_layout.rs` | overwritten on snapshot |

### `smart_replace` / absorb / first-run — remaining holes

1. **Conflicting other includes** — detected, warned, **not imported** (#21 follow-up). Combined with omit-defaults, Nirify cannot win those properties.
2. **Import include order** — not positional (High).
3. **`~/` includes** — niri 26.04 loads them; conflict scan skips; import may skip if outside `~/.config/niri`.
4. **Unparseable `config.kdl`** — backup + replace with minimal include-only file; absorb does **not** import (correct — cannot parse). User must restore from `.nirify-backups/`.
5. **Absorb scalars** — will not update an already-loaded managed file (intentional). Hand-edit of `layout { gaps }` in `config.kdl` after Nirify owns `appearance.kdl` is discarded. Documented; still surprising.
6. **`add_include_line`** — leftover append-only API (unused). Do not wire it.

### Backups / restore

- `smart_replace` and category first-write use atomic write + `config.kdl.backup-TS` / `<file>.kdl.TS.bak`.
- `cleanup_old_backups(10)` groups by first `.kdl` — good.
- Restore resolves `config.kdl.backup-*` → `config.kdl` and `<name>.kdl.*.bak` via `ConfigFile::from_file_name` (works for `blur.kdl`).
- **Gaps:** migration backups invisible in UI; `add_include_line` backup name `config.kdl.backup.TS` also would not restore-match if ever used; no restore of “whole nirify/ tree”; no UI for conflicting-include import from backup.

---

## D) Config god / incomplete list

No `TODO`/`FIXME`/`todo!` in `src/config/`. Half-wired options show up as omit-default, slashdash, or missing fields instead.

### Worst offenders (approx LOC; includes tests)

| File | LOC | Why it hurts |
| --- | ---: | --- |
| `src/ipc/mod.rs` | ~1590 | Socket I/O + JSON types + output helpers + tests in one module. #22 grew `FullOutputInfo` here. |
| `src/app/mod.rs` `App::update` | ~1230 | Not Config-owned, but it is the only caller of absorb/first-run/save/IPC. Any Config fix must navigate this match. |
| `src/config/storage/rules.rs` | ~1190 | Window + layer emit, slashdash, compat gates, large snapshot tests. |
| `src/config/storage/display.rs` | ~1090 | Animations + cursor + overview + **layout override** + outputs. Scale omit lives here. |
| `src/config/storage/mod.rs` | ~880 | `write_all_settings` / `save_dirty` / backup policy / main.kdl compat. |
| `src/config/loader/display.rs` | ~860 | `parse_layout_override` + output + animations. Mirror of the storage god. |
| `src/config/replace.rs` | ~850 | Analyze + rewrite + conflict scan + tests. |
| `src/config/takeover.rs` | ~840 | first-run + absorb + merge + tests. Right place; still one file for two pipelines. |
| `src/config/loader/helpers.rs` | ~840 | Color/CSS parse + file I/O + many helpers. |
| `src/config/loader/rules.rs` | ~800 | Match parse + gradient warn + 26.04 blocks. |
| `src/config/loader/mod.rs` | ~750 | `load_settings_with_result` — long copy-paste `input { device }` blocks. |
| `src/config/loader/import.rs` | ~650 | Document import + include walk + catch-all skip. |
| `src/config/models/keybindings.rs` | ~720 | Settings + 135-entry catalog (catalog is data, not logic). |
| `src/config/storage/input.rs` | ~690 | Six devices + slashdash tablet flags. |
| `src/config/paths.rs` | ~680 | Paths + migrate + add_include + backup cleanup. |
| `src/app/handlers/outputs.rs` | ~530 | Post-#23; Config snapshot + UI rearrange messages. |

### Incomplete / half-wired Config paths (not iced chrome)

- `OutputConfig.scale: f64` cannot represent “unset / auto”.
- Catch-all `window-rule` only as `appearance.corner_radius`.
- `on-xdg-activate` absent (git niri).
- `ConflictingInclude` warn-only — no import API.
- `add_include_line` dead; `HEALTH_CHECK` stale.
- `ConfigPaths` vs `ConfigFile::Blur` inconsistency.
- Import include resolver ≠ `smart_replace` resolver (`~/`, jail, optional).
- `load_keybindings_from_doc` / `doc.get("binds")` first-block only.
- Consolidation `WindowRuleEffectKey` omits 26.04 fields (shadow, popups, background-effect, …) — suggestions can merge rules that differ.

---

## E) Recommended fix order for Robert

No implementation in this PR. Suggested sequence (data-loss first):

1. **`OutputConfig.scale: Option<f64>`** — load/write `None` as omit (auto); `Some(1.0)` as `scale 1.0`. Update #22 snapshot to set `Some(live.scale)`. Add a round-trip test: explicit 1.0 survives save. Unblocks Displays without HiDPI surprise.

2. **Catch-all window-rule import** — stop skipping no-`match` rules in `import_window_rules_from_doc`; keep Appearance radius as a view over a distinguished catch-all **or** stop emitting a second catch-all from `appearance.kdl`. Absorb/first-run tests with `window-rule { opacity 0.9; clip-to-geometry true }`.

3. **Last-wins omits** — emit explicit defaults for properties Nirify claims to own when a `ConflictingInclude` exists **or** always emit `center-focused-column`, `default-column-display`, and `scale` when the category file is managed. Cheapest: always emit the three keywords.

4. **Binds + include order** — import `binds` with last-combo-wins; walk includes **in document order** interleaved with same-file nodes (match niri positionality). One importer, used by first-run, absorb, and keybindings load.

5. **`~/` + optional includes (26.04)** — align `replace::resolve_include_path` with niri 26.04 (`~/` expand); decide whether import jail stays (document it). Parse `optional=true` for conflict scan (missing file ≠ error).

6. **Absorb conflicting includes** — #21 follow-up: offer import of `ConflictingInclude` targets into `nirify/`, then leave the user include or comment it.

7. **#22 VRR on-demand** — do not write `vrr` from live if current setting is `OnDemand` and live `vrr_enabled` is consistent with “sometimes on”.

8. **Output identity** — match live connectors to `output "Make Model Serial"` via IPC make/model/serial (`FullOutputInfo` already has room / tests mention serial). Needed for #23 rearrange on docks.

9. **Fractional layout** — stop `gaps.round() as i32` / `field_f32_as_int` for niri `FloatOrInt` fields.

10. **Rule gradients** — reuse `load_color_or_gradient` in window/layer rule border/focus-ring/tab-indicator (loader already warns).

11. **Hygiene** — delete or `#[deprecated]` `add_include_line`; add `blur_kdl` to `ConfigPaths` **or** stop adding path fields; extend `HEALTH_CHECK`; list migration backups; fix RecentWindows comment; split `storage/display.rs` / `storage/rules.rs`.

12. **Git niri (after 26.04)** — `on-xdg-activate`, `max-bpc`, `disable-10bit-output`, live `map-to-focused-window`.

**Do not:** wire `add_include_line`; emit `map-to-focused-window` as a live node; treat 26.04 `on-xdg-activate` as a 26.04 gap.

---

## Symbols (quick index)

| Symbol | File |
| --- | --- |
| `MANAGED_NODES`, `smart_replace_config`, `analyze_config`, `ConflictingInclude` | `src/config/replace.rs` |
| `first_run_setup`, `absorb_stripped_nodes`, `merge_stripped_into_managed` | `src/config/takeover.rs` |
| `import_from_niri_config_with_result`, `import_from_kdl_str`, `import_window_rules_from_doc` | `src/config/loader/import.rs` |
| `generate_outputs_kdl`, `generate_layout_override_kdl` | `src/config/storage/display.rs` |
| `apply_live_outputs_to_settings` | `src/config/outputs_layout.rs` |
| `generate_main_kdl` | `src/config/storage/behavior.rs` |
| `ConfigFile`, `HEALTH_CHECK` | `src/config/registry.rs` |
| `FeatureCompat`, `NiriFeature::RecentWindows` (25.11) | `src/version.rs` |
| `add_include_line`, `migrate_include_line`, `cleanup_old_backups` | `src/config/paths.rs` |
| `get_full_outputs` | `src/ipc/mod.rs` |
