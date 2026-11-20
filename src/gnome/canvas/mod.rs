pub mod layers;
pub mod painter;
pub mod shapes;
#[cfg(test)]
mod tests;
mod uploads;

use std::cell::RefCell;

use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, graphene};
use trame::render::Scene;
use trame::text::TextSystem;

pub use layers::{build_chrome, build_overlays};
pub use painter::ScenePainter;

type DrawFn = Box<dyn Fn(&gtk::Snapshot, i32, i32, i32)>;
type PickFn = Box<dyn Fn(f64, f64) -> bool>;

pub mod imp {
    use super::*;

    #[derive(Default)]
    pub struct LayerCanvas {
        pub(super) draw: RefCell<Option<DrawFn>>,
        pub(super) pick: RefCell<Option<PickFn>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LayerCanvas {
        const NAME: &'static str = "TrameLayerCanvas";
        type Type = super::LayerCanvas;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for LayerCanvas {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.set_hexpand(true);
            obj.set_vexpand(true);
        }
    }

    impl WidgetImpl for LayerCanvas {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = self.obj();
            if let Some(draw) = self.draw.borrow().as_ref() {
                draw(snapshot, obj.width(), obj.height(), obj.scale_factor());
            }
        }

        fn contains(&self, x: f64, y: f64) -> bool {
            let obj = self.obj();
            let inside =
                x >= 0.0 && y >= 0.0 && x < f64::from(obj.width()) && y < f64::from(obj.height());
            inside && self.pick.borrow().as_ref().is_none_or(|pick| pick(x, y))
        }
    }
}

glib::wrapper! {
    pub struct LayerCanvas(ObjectSubclass<imp::LayerCanvas>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl LayerCanvas {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_draw(&self, draw: DrawFn) {
        self.imp().draw.replace(Some(draw));
    }

    pub fn set_pick(&self, pick: PickFn) {
        self.imp().pick.replace(Some(pick));
    }
}

impl Default for LayerCanvas {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_layer(
    snapshot: &gtk::Snapshot,
    size: (i32, i32, i32),
    painter: &mut ScenePainter,
    scene: &Scene,
    text: &mut TextSystem,
) {
    let (w, h, scale) = (size.0.max(0), size.1.max(0), size.2.max(1));
    if w == 0 || h == 0 || scene.quads().is_empty() {
        return;
    }
    painter.sync_atlas(text.atlas_mut());
    let s = scale as f32;
    let inv = 1.0 / s;
    snapshot.save();
    snapshot.scale(inv, inv);
    let bounds = graphene::Rect::new(0.0, 0.0, w as f32 * s, h as f32 * s);
    let cr = snapshot.append_cairo(&bounds);
    painter.paint(&cr, scene.quads());
    snapshot.restore();
}
