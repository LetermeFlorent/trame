mod helpers;
mod hover;
mod inline;
mod links;
mod paths;

use std::collections::HashMap;

use trame::files::IconStore;
use trame::geometry::Rect;
use trame::locale::Language;
use trame::render::Scene;
use trame::theme::{Appearance, Theme};
use trame::ui::chrome::{PictureFrame, draw_picture, place_picture};
use trame::ui::frame::{FormAction, MessageRequest};
use trame::ui::terminal::preview_rect;
use trame::ui::{FrameSources, FrameView, Motion, UiMetrics, draw_frame};

use self::helpers::quad_drawn_rect;
use crate::paint::paint_with;
use crate::{HEIGHT, WIDTH, frame, text_system, workspace};

#[test]
fn draw_picture_pushes_scrim_picture_and_close_icon_inside_window() {
    let mut text = text_system();
    let layout = frame(&mut text, &workspace());
    let theme = Theme::new(Appearance::Dark);
    let placed = place_picture(layout.bounds, &layout.metrics, 4000, 1000);
    let scene = paint_with(&mut text, &layout, &theme, None, |scene, text, input| {
        draw_picture(scene, text, input, Some(placed));
    });
    let quads = scene.quads();
    assert!(
        quads
            .iter()
            .any(|quad| { quad.params[0] == 0.0 && quad.rect == [0.0, 0.0, WIDTH, HEIGHT] })
    );
    assert!(quads.iter().any(|quad| {
        quad.params[0] == 6.0
            && quad.rect
                == [
                    placed.rect.x,
                    placed.rect.y,
                    placed.rect.width,
                    placed.rect.height,
                ]
    }));
    let close_lines: Vec<_> = quads.iter().filter(|quad| quad.params[0] == 4.0).collect();
    assert_eq!(close_lines.len(), 2);
    for line in close_lines {
        assert!(line.rect[0] > WIDTH / 2.0 && line.rect[0] + line.rect[2] <= WIDTH);
        assert!(line.rect[1] >= 0.0 && line.rect[1] + line.rect[3] <= HEIGHT);
    }
}

#[test]
fn placed_picture_fits_4000x1000_into_800x600_with_margin_and_aspect() {
    let bounds = Rect::new(0.0, 0.0, 800.0, 600.0);
    let metrics = UiMetrics::new(1.0);
    let placed = place_picture(bounds, &metrics, 4000, 1000);
    assert!(placed.rect.width < 800.0 && placed.rect.height < 600.0);
    assert!((placed.rect.width / placed.rect.height - 4.0).abs() < 0.05);
    assert!((placed.rect.x + placed.rect.width / 2.0 - 400.0).abs() <= 1.0);
    assert!((placed.rect.y + placed.rect.height / 2.0 - 300.0).abs() <= 1.0);
}

#[test]
fn preview_box_for_1000x1000_in_8_rows_of_10x20_is_centred_160x160() {
    let rect = preview_rect(Rect::new(0.0, 0.0, 16.0 * 10.0, 8.0 * 20.0), 1000, 1000);
    assert_eq!(rect, Rect::new(0.0, 0.0, 160.0, 160.0));
    let narrow = preview_rect(Rect::new(5.0, 10.0, 100.0, 160.0), 1000, 1000);
    assert_eq!(narrow, Rect::new(5.0, 40.0, 100.0, 100.0));
}

#[test]
fn draw_picture_clips_oversized_frame_to_margin_bounds() {
    let mut text = text_system();
    let layout = frame(&mut text, &workspace());
    let theme = Theme::new(Appearance::Dark);
    let oversized = PictureFrame::centered(layout.bounds, 2000, 1500);
    let scene = paint_with(&mut text, &layout, &theme, None, |scene, text, input| {
        draw_picture(scene, text, input, Some(oversized));
    });
    let pic = scene
        .quads()
        .iter()
        .find(|quad| quad.params[0] == 6.0)
        .expect("picture quad");
    let drawn = quad_drawn_rect(pic.rect, pic.clip);
    assert!(drawn.x >= 32.0 && drawn.y >= 32.0);
    assert!(drawn.right() <= WIDTH - 32.0 && drawn.bottom() <= HEIGHT - 32.0);
}

fn paint_message_and_picture(
    text: &mut trame::text::TextSystem,
    layout: &trame::ui::FrameLayout,
    placed: PictureFrame,
) -> Scene {
    let theme = Theme::new(Appearance::Dark);
    let mut scene = Scene::new(WIDTH, HEIGHT);
    let sources = FrameSources {
        theme: &theme,
        frame: layout,
        view: &FrameView::focused(),
        sessions: &HashMap::new(),
        motion: &Motion::default(),
        drag: None,
        loader: None,
        icons: &IconStore::default(),
        links: None,
        cwd: None,
        picture: Some(placed),
        preview: None,
    };
    draw_frame(&mut scene, text, &sources);
    scene
}

#[test]
fn draw_frame_paints_message_above_picture() {
    let mut text = text_system();
    let mut layout = frame(&mut text, &workspace());
    layout.place_message(
        &mut text,
        &MessageRequest {
            title: "Close",
            lines: &["Confirm?"],
            actions: &[FormAction::Cancel, FormAction::Close],
            strings: Language::English.strings(),
        },
    );
    let popup_rect = layout.message.as_ref().expect("message").popup.rect;
    let placed = place_picture(layout.bounds, &layout.metrics, 4000, 1000);
    let scene = paint_message_and_picture(&mut text, &layout, placed);
    let quads = scene.quads();
    let pic_idx = quads.iter().position(|q| q.params[0] == 6.0).expect("pic");
    let want = [
        popup_rect.x,
        popup_rect.y,
        popup_rect.width,
        popup_rect.height,
    ];
    let msg_idx = quads
        .iter()
        .rposition(|q| q.params[0] == 0.0 && q.rect == want)
        .expect("dialog");
    assert!(msg_idx > pic_idx);
}
