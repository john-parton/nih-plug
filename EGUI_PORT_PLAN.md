# Vizia to egui Port Plan

This document tracks the progress of porting vizia-based plugins to egui.

## Why egui?

After analysis, egui is the better choice for these plugins because:

1. **Simpler custom drawing API** - Direct access to `ui.painter()` instead of implementing canvas programs
2. **Better suited for real-time visualizations** - Immediate mode paradigm fits audio analyzers perfectly
3. **Less boilerplate** - Custom widgets are easier to write
4. **Proven path rendering** - The `epaint` API handles paths, strokes, and fills cleanly
5. **Active ecosystem** - More examples and community support for custom visualizations

## Plugins to Port

### 1. Crisp ✅ **PRIORITY 1** (Simplest)
- **Status**: Not started
- **Complexity**: Low
- **Custom rendering**: None
- **Widgets needed**:
  - Parameter sliders (already available in `nih_plug_egui`)
  - Parameter buttons (already available)
  - Basic layout
- **Estimated effort**: 2-4 hours
- **Notes**: Perfect starting point - only uses standard parameter widgets

### 2. Spectral Compressor 🔶 **PRIORITY 2** (Medium)
- **Status**: Not started
- **Complexity**: Medium
- **Custom rendering**:
  - Spectrum analyzer (vertical bars)
  - Threshold curves (paths/lines)
  - Gain reduction overlay (blended fills)
- **Widgets needed**:
  - Custom analyzer widget with `ui.painter()` drawing
  - Parameter sliders and buttons (already available)
  - Mode toggle button
- **Estimated effort**: 8-12 hours
- **Notes**: 
  - Reference `plugins/spectral_compressor/src/editor/analyzer.rs` for drawing logic
  - The vizia version uses `vg` crate - translate to egui's `epaint` primitives
  - Analyzer has collapsible/expandable mode

### 3. Diopser 🔴 **PRIORITY 3** (Most Complex)
- **Status**: Not started
- **Complexity**: High
- **Custom rendering**:
  - Spectrum analyzer (similar to spectral compressor)
  - **Custom XY pad** with modulation visualization
  - Filter frequency response curves
- **Widgets needed**:
  - Custom spectrum analyzer
  - **Custom XY pad widget** (most complex part)
  - RestrictedParamSlider (custom slider variant)
  - SafeModeButton (custom button)
- **Estimated effort**: 16-24 hours
- **Notes**:
  - XY pad needs drag interaction, modulation display, safe mode clamping
  - Reference `plugins/diopser/src/editor/xy_pad.rs` for interaction logic
  - Safe mode feature restricts parameter ranges

## Implementation Strategy

### Phase 1: Foundation (Crisp)
1. Study `plugins/examples/gain_gui_egui` as reference
2. Port Crisp's simple UI layout
3. Establish patterns for:
   - Window sizing and resizing
   - Parameter layout
   - Consistent styling
4. Test bundling and ensure it works as VST3/CLAP

### Phase 2: Custom Drawing (Spectral Compressor)
1. Create custom analyzer widget using egui's painting API
2. Implement spectrum drawing:
   - Translate `vg::Path` to egui paths
   - Use `ui.painter().add()` for shapes
   - Handle color blending for overlays
3. Implement curve drawing for thresholds
4. Add collapsible analyzer panel
5. Verify real-time performance

### Phase 3: Advanced Widgets (Diopser)
1. Port spectrum analyzer (reuse from Phase 2)
2. Create custom XY pad widget:
   - Drag interaction with `ui.allocate_response()`
   - Draw handle and modulation indicators
   - Implement safe mode parameter clamping
   - Add visual feedback for modulation
3. Port restricted parameter sliders
4. Integrate safe mode button
5. Full testing and refinement

## Technical Notes

### egui Painting Basics
```rust
// Get the painter
let painter = ui.painter();

// Draw a rectangle
painter.rect_filled(rect, rounding, color);

// Draw a path/line
let points = vec![pos1, pos2, pos3];
painter.add(egui::Shape::line(points, stroke));

// Draw with blending
painter.set_clip_rect(rect);
// Draw shapes - egui handles blending based on alpha
```

### Key Differences from Vizia

| Vizia (femtovg) | egui (epaint) | Notes |
|-----------------|---------------|-------|
| `vg::Path` | `egui::Shape::Path` | Similar API |
| `vg::Paint::color()` | `egui::Color32` | egui uses RGBA u8 |
| `canvas.fill_path()` | `painter.add(Shape::Path)` | More direct |
| `canvas.stroke_path()` | `painter.add(Shape::line())` | Simplified |
| CSS styling | Inline Rust styling | Already familiar pattern |

### Resources
- egui docs: https://docs.rs/egui/
- epaint docs: https://docs.rs/epaint/
- Example: `plugins/examples/gain_gui_egui/src/lib.rs`
- nih_plug_egui widgets: `nih_plug_egui/src/widgets/`

## Progress Tracking

- [ ] Crisp ported to egui
- [ ] Spectral Compressor ported to egui
- [ ] Diopser ported to egui
- [ ] All plugins tested in DAW
- [ ] Documentation updated
- [ ] Old vizia versions deprecated/removed (future decision)

## Testing Checklist

For each ported plugin:
- [ ] Builds successfully
- [ ] Standalone version runs
- [ ] VST3 bundle loads in DAW
- [ ] CLAP bundle loads in DAW
- [ ] All parameters work correctly
- [ ] Custom widgets render properly
- [ ] Resizing works (if applicable)
- [ ] Performance is acceptable (no dropped frames)
- [ ] State save/restore works

## Questions/Decisions

1. **Keep both versions?** - TBD: Keep vizia versions or replace them?
2. **Shared widgets?** - Should we create reusable custom widgets in `nih_plug_egui`?
3. **Styling consistency?** - Establish color scheme and spacing constants?
4. **Performance targets?** - Define acceptable FPS for real-time analyzers?

---

**Created**: January 14, 2026
**Branch**: `feature/port-plugins-to-iced-egui` (will rename to `feature/port-plugins-to-egui`)
