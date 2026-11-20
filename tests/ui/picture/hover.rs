use trame::geometry::Rect;
use trame::locale::Language;
use trame::theme::{Appearance, Theme};
use trame::ui::chrome::{
    HoverPreviewFrame, draw_hover_preview, draw_picture, picture_close_rect, place_hover_preview,
    place_picture, place_picture_open,
};

use crate::paint::paint_with;
use crate::{HEIGHT, WIDTH, frame, text_system, workspace};

#[test]
fn draw_picture_with_open_button_places_button_left_of_close_inside_window() {
    let mut text = text_system();
    let layout = frame(&mut text, &workspace());
    let theme = Theme::new(Appearance::Dark);
    let label = Language::English.strings().open_in_viewer;
    let open = place_picture_open(
        &mut text,
        layout.bounds,
        layout.title_bar.rect.bottom(),
        &layout.metrics,
        label,
    )
    .expect("open button rect");
    let close = picture_close_rect(
        layout.bounds,
        layout.title_bar.rect.bottom(),
        &layout.metrics,
    );
    assert!(open.right() < close.x);
    assert_eq!(open.y, close.y);
    assert_eq!(open.height, close.height);
    assert!(open.x > 0.0);

    let placed =
        place_picture(layout.bounds, &layout.metrics, 400, 300).with_open(Some(open), Some(label));
    let scene = paint_with(&mut text, &layout, &theme, None, |scene, text, input| {
        draw_picture(scene, text, input, Some(placed));
    });
    let quads = scene.quads();
    assert!(
        quads
            .iter()
            .any(|q| { q.params[0] == 0.0 && q.rect == [open.x, open.y, open.width, open.height] })
    );
    assert!(quads.iter().any(|q| q.params[0] == 2.0));
}

#[test]
fn open_button_fits_full_french_and_english_labels_without_truncation() {
    let mut text = text_system();
    let layout = frame(&mut text, &workspace());
    let pad_x = layout.metrics.px(10.0);
    for lang in [Language::English, Language::French] {
        let label = lang.strings().open_in_viewer;
        let open = place_picture_open(
            &mut text,
            layout.bounds,
            layout.title_bar.rect.bottom(),
            &layout.metrics,
            label,
        )
        .expect("open button rect");
        let full = text.layout(label, layout.metrics.label()).width;
        assert!(
            open.width - 2.0 * pad_x >= full,
            "{label}: {open:?} < {full}"
        );
    }
}

#[test]
fn draw_hover_preview_places_card_above_or_below_anchor_inside_window() {
    let mut text = text_system();
    let layout = frame(&mut text, &workspace());
    let theme = Theme::new(Appearance::Dark);
    let title_bottom = layout.title_bar.rect.bottom();

    let low_anchor = Rect::new(40.0, HEIGHT - 40.0, 200.0, 20.0);
    let (card_above, _) = place_hover_preview(
        layout.bounds,
        title_bottom,
        low_anchor,
        &layout.metrics,
        (120, 80),
    );
    assert!(card_above.bottom() <= low_anchor.y);
    assert!(card_above.y >= title_bottom);

    let high_anchor = Rect::new(WIDTH - 40.0, title_bottom + 4.0, 200.0, 20.0);
    let (card_below, _) = place_hover_preview(
        layout.bounds,
        title_bottom,
        high_anchor,
        &layout.metrics,
        (120, 80),
    );
    assert!(card_below.y >= high_anchor.bottom());
    assert!(card_below.right() <= WIDTH);

    let pixels = vec![200u8; 40 * 30 * 4];
    let preview = HoverPreviewFrame {
        job: 99,
        anchor: low_anchor,
        size: (40, 30),
        pixels: &pixels,
    };
    let scene = paint_with(&mut text, &layout, &theme, None, |scene, text, input| {
        draw_hover_preview(scene, text, input, Some(preview));
    });
    assert!(scene.quads().iter().any(|q| q.params[0] == 3.0));
}
