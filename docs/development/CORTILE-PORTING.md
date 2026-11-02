# Cortile porting notes

Reference project: https://github.com/leukipp/cortile

Cortile is an X11 auto-tiling manager written in Go and licensed under the MIT License. Its README documents workspace-based tiling, panel detection, vertical/horizontal/maximized/fullscreen modes, remembered proportions, floating/sticky windows, drag-and-drop swaps, multi-monitor behavior, keyboard/hot-corner/tray bindings and a UI for tiling mode.

## Porting policy

1. Port behavior intentionally rather than translating Go syntax mechanically.
2. Keep the domain/layout engine independent of X11, GTK and IPC.
3. For every copied or adapted source file, record the original path, copyright and applicable license text.
4. Preserve Cortile's MIT copyright and permission notices in source distributions that include substantial portions of its code.
5. Review dependency licenses independently; Cortile's license does not automatically cover third-party dependencies.
6. Add parity fixtures for externally observable layout behavior before declaring a port complete.
7. Where K-tile changes behavior for safety or Cinnamon integration, document the divergence and add regression tests.

## Initial behavior inventory

- Master/stack layouts: vertical left/right and horizontal top/bottom.
- Maximized and fullscreen modes.
- Workspace-specific layout state and remembered proportions.
- Floating and sticky window handling.
- Panel/workarea detection.
- Monitor-aware tiling.
- Keyboard, mouse, hot-corner and tray interactions.

This list is a source-derived inventory, not evidence that K-tile currently implements those features.

## Reviewed upstream source paths

The following upstream files were inspected as behavioral references on the upstream `main` branch. They have **not** been copied into K-tile:

| Upstream path | Behavior to port or compare |
|---|---|
| `layout/vertical.go` | Vertical master/stack placement, gap arithmetic, proportions, master/slave resizing behavior |
| `layout/horizontal.go` | Horizontal master/stack placement, gap arithmetic, proportions, master/slave resizing behavior |
| `layout/maximized.go` | Shared-area layout with window gaps and minimum-size constraints |
| `layout/fullscreen.go` | Fullscreen layout semantics; K-tile must continue excluding already-fullscreen windows from ordinary tiling |
| `desktop/layout.go` | Expected layout operations: client insertion/removal, master promotion, swaps, focus traversal and proportion changes |
| `desktop/workspace.go` | Workspace/monitor scoped layouts, enable/disable tiling, state persistence, restore and cycle behavior |
| `common/config.go` | TOML configuration keys, runtime reload behavior and configuration defaults |

The inspected upstream code uses MIT-licensed Cortile source. Preserve the original notices for any substantial copied/adapted implementation and add file-level notes when a port is actually made. Porting a behavior is not the same as copying its implementation; newly authored Rust equivalents should be identified as such.

## Attribution

Cortile repository license: https://github.com/leukipp/cortile/blob/main/LICENSE

The upstream license includes copyright notices for Berin Larson (2019) and leukipp (2022). Before copying code, retain those notices as required and document exact source paths and modifications. The initial K-tile foundation in this branch is newly authored Rust code and does not claim a completed source-code port.
