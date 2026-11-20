use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{cairo, gdk};
use trame::theme::{Rgba, Theme};

pub const PICTURE_MARGIN: i32 = 32;
pub const CLOSE_SIZE: i32 = 28;
pub const CLOSE_PAD: i32 = 12;
pub const OPEN_GAP: i32 = 8;
const CLOSE_ICON_SIZE: i32 = 14;
const ICON_INSET: f64 = 3.5;
const ICON_STROKE: f64 = 1.6;
const SCRIM_ALPHA: f32 = 0.72;
const BUTTON_ALPHA: f32 = 0.9;
const HOVER_BLEND: f32 = 0.35;
const BUTTON_RADIUS: i32 = 5;
const OPEN_PAD_X: i32 = 10;
const NORMAL_WEIGHT: u16 = 400;
const FULL_CHANNEL: f32 = 255.0;

pub fn close_icon(ink: Rc<Cell<Rgba>>) -> gtk::DrawingArea {
    let area = gtk::DrawingArea::new();
    area.set_content_width(CLOSE_ICON_SIZE);
    area.set_content_height(CLOSE_ICON_SIZE);
    area.set_halign(gtk::Align::Center);
    area.set_valign(gtk::Align::Center);
    area.set_draw_func(move |_, cr, width, height| {
        let color = ink.get();
        let (w, h) = (f64::from(width), f64::from(height));
        cr.set_source_rgba(
            f64::from(color.r),
            f64::from(color.g),
            f64::from(color.b),
            f64::from(color.a),
        );
        cr.set_line_width(ICON_STROKE);
        cr.set_line_cap(cairo::LineCap::Round);
        cr.move_to(ICON_INSET, ICON_INSET);
        cr.line_to(w - ICON_INSET, h - ICON_INSET);
        cr.move_to(w - ICON_INSET, ICON_INSET);
        cr.line_to(ICON_INSET, h - ICON_INSET);
        let _ = cr.stroke();
    });
    area
}

pub fn install_provider() -> gtk::CssProvider {
    let provider = gtk::CssProvider::new();
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    provider
}

pub fn overlay_css(theme: &Theme) -> String {
    let scrim = css_rgba(Rgba::BLACK.with_alpha(SCRIM_ALPHA));
    let base = css_rgba(theme.chrome.with_alpha(BUTTON_ALPHA));
    let hover = css_rgba(
        theme
            .chrome
            .mix(theme.hover, HOVER_BLEND)
            .with_alpha(BUTTON_ALPHA),
    );
    let border = css_rgba(theme.border_strong);
    let text = css_rgba(theme.text);
    format!(
        ".trame-picture-scrim {{ background-color: {scrim}; }}\n\
         button.trame-picture-btn {{\n\
             background: {base};\n\
             background-image: none;\n\
             color: {text};\n\
             border: 1px solid {border};\n\
             border-radius: {BUTTON_RADIUS}px;\n\
             min-height: {CLOSE_SIZE}px;\n\
             min-width: {CLOSE_SIZE}px;\n\
             padding: 0;\n\
             box-shadow: none;\n\
         }}\n\
         button.trame-picture-btn:hover {{\n\
             background: {hover};\n\
             background-image: none;\n\
         }}\n\
         button.trame-picture-open {{\n\
             padding: 0 {OPEN_PAD_X}px;\n\
             font-weight: {NORMAL_WEIGHT};\n\
         }}"
    )
}

fn css_rgba(color: Rgba) -> String {
    let r = (color.r * FULL_CHANNEL).round() as u8;
    let g = (color.g * FULL_CHANNEL).round() as u8;
    let b = (color.b * FULL_CHANNEL).round() as u8;
    format!("rgba({r}, {g}, {b}, {:.3})", color.a)
}
