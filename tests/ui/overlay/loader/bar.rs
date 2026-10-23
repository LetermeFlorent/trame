use trame::render::Scene;
use trame::theme::{Appearance, Theme};
use trame::ui::chrome::{LoaderProgress, LoaderView};

use super::{paint, waiting};
use crate::{frame, text_system, workspace};

const TRACK_ALPHA: f32 = 0.2;

fn bar(scene: &Scene, theme: &Theme) -> ([f32; 4], Option<[f32; 4]>) {
    let track = theme.text_muted.with_alpha(TRACK_ALPHA).to_array();
    let track = scene
        .quads()
        .iter()
        .find(|quad| quad.color == track)
        .expect("the track")
        .rect;
    let fill = scene
        .quads()
        .iter()
        .find(|quad| quad.color == theme.accent.to_array() && quad.rect[1] == track[1])
        .map(|quad| quad.rect);
    (track, fill)
}

#[test]
fn the_progress_bar_fills_its_share_of_the_track() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let layout = frame(&mut text, &workspace());
    let loader = LoaderView {
        progress: Some(LoaderProgress {
            fraction: 0.25,
            label: "25%",
        }),
        ..waiting(1.0)
    };
    let (track, fill) = bar(&paint(&mut text, &layout, &theme, &loader), &theme);
    let fill = fill.expect("the filled part");
    assert_eq!(fill[0], track[0]);
    assert!((fill[2] - track[2] / 4.0).abs() < 0.01);
}

#[test]
fn the_waiting_sweep_stays_inside_the_track() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let layout = frame(&mut text, &workspace());
    for step in 0..30 {
        let loader = LoaderView {
            spin: step as f32 * 0.1,
            ..waiting(1.0)
        };
        let (track, fill) = bar(&paint(&mut text, &layout, &theme, &loader), &theme);
        if let Some(fill) = fill {
            assert!(fill[0] >= track[0]);
            assert!(fill[0] + fill[2] <= track[0] + track[2] + 0.01);
        }
    }
}
