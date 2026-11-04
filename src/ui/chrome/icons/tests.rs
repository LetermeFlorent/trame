use super::lucide::Glyph;
use super::outline::{Shape, build, outline};
use super::path::flatten;
use super::{Icon, glyph};
use crate::geometry::Point;
use crate::theme::{Emblem, Letters};

fn close(a: Point, b: Point) -> bool {
    (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3
}

fn last(lines: &[Vec<Point>]) -> Point {
    *lines.last().and_then(|line| line.last()).expect("a point")
}

#[test]
fn absolute_and_relative_commands_reach_the_same_points() {
    let absolute = flatten("M2 3L5 3H9V7L2 7");
    let relative = flatten("m2 3l3 0h4v4l-7 0");
    assert_eq!(absolute.len(), 1);
    assert_eq!(absolute, relative);
    let expected = [(2.0, 3.0), (5.0, 3.0), (9.0, 3.0), (9.0, 7.0), (2.0, 7.0)];
    for (point, (x, y)) in absolute[0].iter().zip(expected) {
        assert!(close(*point, Point::new(x, y)));
    }
}

#[test]
fn implicit_repeats_and_packed_numbers_are_read() {
    let lines = flatten("m16 15-3-3 3-3");
    assert_eq!(lines[0].len(), 3);
    assert!(close(lines[0][2], Point::new(16.0, 9.0)));
    let packed = flatten("M0 0l.5.5-1.25-.75");
    assert!(close(last(&packed), Point::new(-0.75, -0.25)));
}

#[test]
fn arcs_end_exactly_at_their_end_point() {
    for (d, x, y) in [
        ("M12 2a14.5 14.5 0 0 0 0 20", 12.0, 22.0),
        ("M3 5A9 3 0 0 0 21 5", 21.0, 5.0),
        ("M12 20a8 8 0 1 0 0-16", 12.0, 4.0),
        ("M0 0a1 1 0 0 1 10 0", 10.0, 0.0),
    ] {
        let lines = flatten(d);
        assert!(close(last(&lines), Point::new(x, y)), "{d}");
        assert!(lines[0].len() > 3, "{d} is flattened into several segments");
    }
}

#[test]
fn an_arc_bulges_on_its_sweep_side() {
    let lines = flatten("M12 20a8 8 0 1 0 0-16");
    let leftmost = lines[0]
        .iter()
        .map(|point| point.x)
        .fold(f32::MAX, f32::min);
    let rightmost = lines[0]
        .iter()
        .map(|point| point.x)
        .fold(f32::MIN, f32::max);
    assert!(leftmost > 11.9 && (rightmost - 20.0).abs() < 0.1);
}

#[test]
fn z_closes_back_to_the_subpath_start() {
    let lines = flatten("M1 1h4v4zm10 0h2v2Z");
    assert_eq!(lines.len(), 2);
    assert!(close(*lines[0].last().unwrap(), Point::new(1.0, 1.0)));
    assert!(close(lines[1][0], Point::new(11.0, 1.0)));
    assert!(close(*lines[1].last().unwrap(), Point::new(11.0, 1.0)));
}

#[test]
fn curves_end_at_their_end_point() {
    let cubic = flatten("M4 16c-1.1 0-2-.9-2-2");
    assert!(close(last(&cubic), Point::new(2.0, 14.0)));
    let smooth = flatten("M0 0C1 2 3 2 4 0S7-2 8 0");
    assert!(close(last(&smooth), Point::new(8.0, 0.0)));
    let quad = flatten("M0 0Q2 4 4 0T8 0");
    assert!(close(last(&quad), Point::new(8.0, 0.0)));
}

#[test]
fn glyph_order_matches_the_cache_index() {
    for (index, glyph) in Glyph::ALL.into_iter().enumerate() {
        assert_eq!(glyph as usize, index);
    }
}

#[test]
fn every_icon_and_emblem_has_lucide_data() {
    let letters = Letters::parse("AB").expect("two letters");
    let mut icons = vec![
        Icon::Sidebar { open: true },
        Icon::Sidebar { open: false },
        Icon::Plus,
        Icon::Close,
        Icon::SplitRight,
        Icon::Minimize,
        Icon::Maximize,
        Icon::Restore,
        Icon::Terminal,
        Icon::Gear,
        Icon::Update,
        Icon::Pencil,
        Icon::Ban,
        Icon::Emblem(Emblem::Letters(letters)),
        Icon::Emblem(Emblem::Image(1)),
    ];
    icons.extend(Emblem::ALL.map(Icon::Emblem));
    for icon in icons {
        let glyph = glyph(icon);
        assert!(!glyph.source().is_empty(), "{icon:?}");
        assert_eq!(outline(glyph), build(glyph).as_slice(), "{icon:?}");
        assert!(outline(glyph).iter().all(|shape| match shape {
            Shape::Stroke(points) => points.len() >= 2,
            _ => true,
        }));
    }
    for glyph in Glyph::ALL {
        assert!(!glyph.source().is_empty(), "{glyph:?}");
    }
}

#[test]
fn every_preset_emblem_draws_a_distinct_glyph() {
    let glyphs: Vec<Glyph> = Emblem::ALL
        .map(|emblem| glyph(Icon::Emblem(emblem)))
        .to_vec();
    for (index, glyph) in glyphs.iter().enumerate() {
        assert!(!glyphs[index + 1..].contains(glyph));
    }
}
