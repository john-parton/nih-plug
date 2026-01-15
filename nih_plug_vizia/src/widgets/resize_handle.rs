//! A resize handle for uniformly scaling a plugin GUI.

use vizia::prelude::*;
use vizia::vg;

use super::GuiContextEvent;

/// A resize handle placed at the bottom right of the window that lets you resize the window.
///
/// Needs to be the last element in the GUI because of how event targetting in Vizia works right
/// now.
pub struct ResizeHandle {
    /// Will be set to `true` if we're dragging the parameter. Resetting the parameter or entering a
    /// text value should not initiate a drag.
    drag_active: bool,

    /// The scale factor when we started dragging. This is kept track of separately to avoid
    /// accumulating rounding errors.
    start_scale_factor: f64,
    /// The DPI factor when we started dragging, includes both the HiDPI scaling and the user
    /// scaling factor. This is kept track of separately to avoid accumulating rounding errors.
    start_dpi_factor: f32,
    /// The cursor position in physical screen pixels when the drag started.
    start_physical_coordinates: (f32, f32),
}

impl ResizeHandle {
    /// Create a resize handle at the bottom right of the window. This should be created at the top
    /// level. Dragging this handle around will cause the window to be resized.
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        // Styling is done in the style sheet, but positioning needs inline modifiers in Vizia 3
        ResizeHandle {
            drag_active: false,
            start_scale_factor: 1.0,
            start_dpi_factor: 1.0,
            start_physical_coordinates: (0.0, 0.0),
        }
        .build(cx, |_| {})
        .position_type(PositionType::Absolute)
        .bottom(Pixels(0.0))
        .right(Pixels(0.0))
    }
}

impl View for ResizeHandle {
    fn element(&self) -> Option<&'static str> {
        Some("resize-handle")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match *window_event {
            WindowEvent::MouseDown(MouseButton::Left) => {
                // The handle is a triangle, so we should also interact with it as if it was a
                // triangle
                if intersects_triangle(
                    cx.cache.get_bounds(cx.current()),
                    (cx.mouse().cursor_x, cx.mouse().cursor_y),
                ) {
                    cx.capture();
                    cx.set_active(true);

                    self.drag_active = true;
                    self.start_scale_factor = cx.scale_factor() as f64;
                    self.start_dpi_factor = cx.scale_factor();
                    self.start_physical_coordinates = (
                        cx.mouse().cursor_x * self.start_dpi_factor,
                        cx.mouse().cursor_y * self.start_dpi_factor,
                    );

                    meta.consume();
                } else {
                    // TODO: The click should be forwarded to the element behind the triangle
                }
            }
            WindowEvent::MouseUp(MouseButton::Left) => {
                if self.drag_active {
                    cx.release();
                    cx.set_active(false);

                    self.drag_active = false;
                }
            }
            WindowEvent::MouseMove(x, y) => {
                cx.set_hover(intersects_triangle(
                    cx.cache.get_bounds(cx.current()),
                    (x, y),
                ));

                if self.drag_active {
                    // We need to convert our measurements into physical pixels relative to the
                    // initial drag to be able to keep a consistent ratio. This 'relative to the
                    // start' bit is important because otherwise we would be comparing the position
                    // to the same absolute screen position.
                    // TODO: This may start doing fun things when the window grows so large that it
                    //       gets pushed upwards or leftwards
                    let (compensated_physical_x, compensated_physical_y) =
                        (x * self.start_dpi_factor, y * self.start_dpi_factor);
                    let (start_physical_x, start_physical_y) = self.start_physical_coordinates;
                    let new_scale_factor = (self.start_scale_factor
                        * (compensated_physical_x / start_physical_x)
                            .max(compensated_physical_y / start_physical_y)
                            as f64)
                        // Vizia rounds borders to integer pixels, and at <0.5 scaling one pixel
                        // borders will simply disappear
                        .max(0.5);

                    // Emit the SetScale event to update the scale factor
                    cx.emit(GuiContextEvent::SetScale(new_scale_factor));
                }
            }
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        // We'll draw the handle directly as styling elements for this is going to be a bit tricky

        // These basics are taken directly from the default implementation of this function
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let background_color: vg::Color = cx.background_color().into();
        let border_color: vg::Color = cx.border_color().into();
        let border_width = cx.border_width();

        // Draw background rectangle
        let x = bounds.x + border_width / 2.0;
        let y = bounds.y + border_width / 2.0;
        let w = bounds.w - border_width;
        let h = bounds.h - border_width;

        let mut bg_path = vg::Path::new();
        bg_path.move_to((x, y));
        bg_path.line_to((x, y + h));
        bg_path.line_to((x + w, y + h));
        bg_path.line_to((x + w, y));
        bg_path.close();

        // Fill with background color
        let mut bg_paint = vg::Paint::default();
        bg_paint.set_color(background_color);
        bg_paint.set_style(vg::paint::Style::Fill);
        canvas.draw_path(&bg_path, &bg_paint);

        // Draw border if width > 0
        if border_width > 0.0 {
            let mut border_paint = vg::Paint::default();
            border_paint.set_color(border_color);
            border_paint.set_style(vg::paint::Style::Stroke);
            border_paint.set_stroke_width(border_width);
            canvas.draw_path(&bg_path, &border_paint);
        }

        // Draw a simple triangle for the resize handle
        let mut triangle_path = vg::Path::new();
        triangle_path.move_to((x, y + h));
        triangle_path.line_to((x + w, y + h));
        triangle_path.line_to((x + w, y));
        triangle_path.close();

        let font_color: vg::Color = cx.font_color().into();
        let mut triangle_paint = vg::Paint::default();
        triangle_paint.set_color(font_color);
        triangle_paint.set_style(vg::paint::Style::Fill);
        canvas.draw_path(&triangle_path, &triangle_paint);
    }
}

/// Test whether a point intersects with the triangle of this resize handle.
fn intersects_triangle(bounds: BoundingBox, (x, y): (f32, f32)) -> bool {
    // We could also compute Barycentric coordinates, but this is simple and I like not having to
    // think. Just check if (going clockwise), the point is on the right of each of all of the
    // triangle's edges. We can compute this using the determinant of the 2x2 matrix formed by two
    // column vectors, aka the perp dot product, aka the wedge product.
    // NOTE: Since this element is positioned in the bottom right corner we would technically only
    //       have to calculate this for `v1`
    let (p1x, p1y) = bounds.bottom_left();
    let (p2x, p2y) = bounds.top_right();
    // let (p3x, p3y) = bounds.bottom_right();

    let (v1x, v1y) = (p2x - p1x, p2y - p1y);
    // let (v2x, v2y) = (p3x - p2x, p3y - p2y);
    // let (v3x, v3y) = (p1x - p3x, p1y - p3y);

    ((x - p1x) * v1y) - ((y - p1y) * v1x) >= 0.0
    // && ((x - p2x) * v2y) - ((y - p2y) * v2x) >= 0.0
    // && ((x - p3x) * v3y) - ((y - p3y) * v3x) >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_intersection() {
        let bbox = BoundingBox {
            x: 10.0,
            y: 10.0,
            w: 10.0,
            h: 10.0,
        };

        assert!(!intersects_triangle(bbox, (10.0, 10.0)));
        assert!(intersects_triangle(bbox, (20.0, 10.0)));
        assert!(intersects_triangle(bbox, (10.0, 20.0)));
        assert!(intersects_triangle(bbox, (20.0, 20.0)));

        assert!(intersects_triangle(bbox, (15.0, 15.0)));
        assert!(!intersects_triangle(bbox, (14.9, 15.0)));
        assert!(!intersects_triangle(bbox, (15.0, 14.9)));
    }
}
