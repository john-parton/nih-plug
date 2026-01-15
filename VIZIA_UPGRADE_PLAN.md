# NIH-plug Vizia 3 Migration Guide

## Status
- ✅ nih_plug_vizia wrapper complete
- ✅ Canvas drawing API ported
- ✅ Plugins: gain_gui_vizia, crisp, diopser, spectral_compressor
- ⏳ Layout fixes in progress

**Branch:** Using vizia `scale` branch (PR #607) for `WindowEvent::SetUserScale` support

## Documentation
- Docs: https://docs.vizia.dev/
- Book: https://book.vizia.dev/
- GitHub: https://github.com/vizia/vizia

## 🔥 Critical Design Decision: Rust-First, Minimal CSS

**After extensive testing, we've determined the best approach:**

### ✅ DO: Use Rust Inline Modifiers
- **Compile-time safety** - IDE catches errors immediately
- **No runtime CSS warnings** - Avoid custom property warnings
- **Explicit and clear** - What you see is what you get
- **Better refactoring** - Rust's type system helps

### ❌ DON'T: Rely on CSS for Layout
- CSS has **many unsupported properties** in morphorm
- CSS warnings only appear at **runtime** (too late!)
- `text-align`, `line-height` may not work as expected
- `child-*` properties completely removed in Vizia 3

### Strategy Going Forward
1. **Remove CSS classes** from widget code where possible
2. **Inline all styling** in Rust using modifiers
3. **Create reusable Rust components** instead of CSS classes
4. **Keep CSS only for**:
   - Global theme colors
   - Absolutely necessary base styles
   - Properties confirmed to work in morphorm

## Critical: Vizia CSS vs Web CSS

Vizia uses **morphorm layout engine**, NOT standard CSS.

### Property Renames
| Web CSS / Vizia 2 | Vizia 3 |
|-------------------|---------|
| `border-radius` | `corner-radius` |
| `flex-direction` | `layout-type` |
| `position` | `position-type` |
| `font-size: 15` | `font-size: 15px` (units required) |

### CSS Limitations & Gotchas
- ❌ `position: absolute` in CSS ignored - use `.position_type(PositionType::Absolute)` modifier
- ❌ Vizia 2 properties removed: `child-space`, `child-top`, `child-bottom`, `child-left`, `child-right`
- ❌ `text-align` may not work without proper width (use `.width(Stretch(1.0))`)
- ❌ `line-height` triggers runtime warnings - unsupported
- ✅ Use inline modifiers: `.corner_radius()`, `.translate()`, `.text_align()`, `.background_color()`
- ✅ Flexible spacing: `gap: 1s` in CSS = `Stretch(1.0)` in Rust

## Techniques

### Horizontal Centering with Alignment
Use `.alignment(Alignment::TopCenter)` on stacks for clean centering:

```rust
HStack::new(cx, |cx| {
    Label::new(cx, "Centered Text");
})
.alignment(Alignment::TopCenter);
```

### Center-Based Absolute Positioning
For draggable handles where center should align to position:

```rust
Element::new(cx)
    .position_type(PositionType::Absolute)
    .top(y_position_lens)
    .left(x_position_lens)
    .translate((Pixels(-10.0), Pixels(-10.0)))  // -half width, -half height
    .pointer_events(PointerEvents::None)
    .hoverable(false)
```

Key: `.translate()` offsets render position without affecting layout.

### Canvas Drawing API (Vizia 3)
Working pattern using skia-safe:

```rust
// Paint setup
let mut paint = vg::Paint::default();
paint.set_color(color);
paint.set_style(vg::paint::Style::Stroke);
paint.set_stroke_width(width);

// Path creation
let mut path = vg::Path::new();
path.move_to((x1, y1));  // Tuple syntax
path.line_to((x2, y2));

// Drawing
canvas.draw_path(&path, &paint);  // NOT stroke_path()
```

Key changes:
- ❌ Old: `vg::Paint::color(color).with_line_width(width)`
- ✅ New: `vg::Paint::default()` + setters
- ❌ Old: `canvas.stroke_path()`
- ✅ New: `canvas.draw_path()`

### Periodic Repaints (Animated Views)
Use background thread with `ContextProxy`:

```rust
const REPAINT_INTERVAL: Duration = Duration::from_millis(30);

pub enum MyViewEvent { Repaint }

impl MyAnimatedView {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self { /* ... */ }
            .build(cx, |_cx| ())
            .on_build(|cx| {
                cx.spawn(move |proxy| loop {
                    std::thread::sleep(REPAINT_INTERVAL);
                    if proxy.emit(MyViewEvent::Repaint).is_err() { break; }
                });
            })
    }
}

impl View for MyAnimatedView {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            MyViewEvent::Repaint => cx.needs_redraw(),
        });
    }
    
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        // Custom drawing - called every ~30ms
    }
}
```

## API Migration Reference

### Universal Fixes (All Plugins)
1. Font: `FamilyOwned::Name` → `FamilyOwned::Named`
2. Remove: `.child_top()`, `.child_bottom()`, `.child_left()`, `.child_right()`, `.child_space()`
3. Spacing: `.row_between(Pixels(n))` → `.horizontal_gap(Pixels(n))`
4. ScrollView: Use 2-parameter constructor `ScrollView::new(cx, |cx| { ... })`
5. ResizeHandle: Use fully qualified `nih_plug_vizia::widgets::ResizeHandle`
6. Mouse: `cursorx`/`cursory` → `cursor_x`/`cursor_y`
7. PositionType: `SelfDirected` → `Absolute`

### Wrapper Changes
- Removed: baseview dependency, `TextConfig`, `vizia::fonts`
- Window config: Remove `.with_text_config()`
- Window size: `cx.window_size()` → `cx.bounds()`
- Set size: `cx.set_window_size()` → `cx.set_width()` + `cx.set_height()`
- Scale: `cx.user_scale_factor()` → `cx.scale_factor()` (f32, read-only)
- User scale control: Use `WindowEvent::SetUserScale(f32)` from scale branch

### Font System
- Built-in fonts (Roboto, Tabler Icons) removed
- Applications must provide fonts or use system fonts
- Font registration functions are empty stubs

## Completed Migrations

### nih_plug_vizia Wrapper ✅
- editor.rs: Removed `.with_text_config()`, updated window size API
- vizia_assets.rs: Removed font exports, stubbed registration
- widgets.rs: Updated window sizing, scale factor (f32, read-only)
- param_slider.rs: Removed layout methods, updated mouse coords
- peak_meter.rs, resize_handle.rs: Disabled drawing (awaiting port)

### Plugins ✅
- **gain_gui_vizia**: All API updates applied
- **crisp**: All API updates applied
- **diopser**: All API updates + canvas drawing + periodic repaints working

## Known Issues

### Text Alignment in morphorm
Labels may appear left-aligned in flexible layouts. CSS `text-align` not fully supported. Workaround: Accept left alignment or use alternative layout structures.

## Next Steps
1. Port spectral_compressor canvas drawing
2. Implement peak_meter/resize_handle custom drawing
3. Test remaining plugins: soft_vacuum, loudness_war_winner, safety_limiter, buffr_glitch
