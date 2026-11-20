use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, pango};

use super::TermView;

const INTERFACE_SCHEMA: &str = "org.gnome.desktop.interface";
const MONOSPACE_KEY: &str = "monospace-font-name";
const FALLBACK_FONT: &str = "Monospace";
const SAMPLE: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Clone)]
pub struct Metrics {
    pub width: f32,
    pub height: f32,
    pub font: pango::FontDescription,
}

pub fn current(view: &TermView) -> Metrics {
    let imp = view.imp();
    if let Some(metrics) = imp.metrics.borrow().as_ref() {
        return metrics.clone();
    }
    let metrics = measure(view, imp.font_size.get());
    imp.metrics.replace(Some(metrics.clone()));
    metrics
}

pub fn set_font_size(view: &TermView, size: f32) {
    let imp = view.imp();
    if (imp.font_size.get() - size).abs() > 0.05 {
        imp.font_size.set(size);
        imp.metrics.replace(None);
        view.queue_draw();
    }
}

fn measure(view: &TermView, size: f32) -> Metrics {
    let mut font = pango::FontDescription::from_string(&font_name());
    font.set_absolute_size((size.max(6.0) * pango::SCALE as f32).round() as f64);
    let layout = view.create_pango_layout(Some(SAMPLE));
    layout.set_font_description(Some(&font));
    let (width, height) = layout.size();
    let scale = pango::SCALE as f32;
    Metrics {
        width: width as f32 / scale / SAMPLE.len() as f32,
        height: (height as f32 / scale).ceil(),
        font,
    }
}

fn font_name() -> String {
    let known = gio::SettingsSchemaSource::default()
        .and_then(|source| source.lookup(INTERFACE_SCHEMA, true))
        .is_some();
    if !known {
        return FALLBACK_FONT.into();
    }
    let name = gio::Settings::new(INTERFACE_SCHEMA).string(MONOSPACE_KEY);
    if name.is_empty() {
        FALLBACK_FONT.into()
    } else {
        name.into()
    }
}
