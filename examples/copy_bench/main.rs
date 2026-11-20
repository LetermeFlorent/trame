mod cases;
mod screens;

use std::hint::black_box;
use std::time::Instant;

use cases::{CASES, Case};
use trame::geometry::Point;
use trame::render::Scene;
use trame::session::{MAX_TRANSCRIPT_BYTES, TranscriptCopy};
use trame::term::Terminal;
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};
use trame::theme::{Appearance, Theme};
use trame::ui::terminal::{ScreenCopy, TerminalPaint, draw_terminal};

const FRAME_BUDGET_US: f64 = 16_700.0;
const MICROS_PER_SECOND: f64 = 1_000_000.0;
const FONT_SIZE: f32 = 14.0;
const ROUNDS: usize = 10;
const COPY_ITERS: usize = 200;
const WORK_ITERS: usize = 10;

struct PaintTarget {
    terminal: Terminal,
    copy: ScreenCopy,
}

impl PaintTarget {
    fn new(rows: usize, cols: usize) -> Self {
        let terminal = screens::marked(rows, cols);
        let copy = ScreenCopy::of(terminal.screen(), 0.0);
        Self { terminal, copy }
    }
}

struct Context {
    small: Terminal,
    large: Terminal,
    marked: PaintTarget,
    wide: PaintTarget,
    scene: Scene,
    text: TextSystem,
    theme: Theme,
}

impl Context {
    fn new() -> Self {
        let marked = PaintTarget::new(cases::LARGE_ROWS, cases::LARGE_COLS);
        let wide = PaintTarget::new(cases::MARK_ROWS, cases::MARK_COLS);
        let mut text = TextSystem::new(FONT_SIZE, DEFAULT_LINE_HEIGHT).expect("a monospace font");
        let theme = Theme::new(Appearance::Dark);
        let (w, h) = scene_size(&text, &marked.terminal);
        let mut scene = Scene::new(w, h);
        paint_once(&mut scene, &mut text, &theme, &marked);
        paint_once(&mut scene, &mut text, &theme, &wide);
        Self {
            small: screens::styled(cases::SMALL_ROWS, cases::SMALL_COLS),
            large: screens::styled(cases::LARGE_ROWS, cases::LARGE_COLS),
            marked,
            wide,
            scene,
            text,
            theme,
        }
    }

    fn run(&mut self, case: Case) -> f64 {
        let target = match case {
            Case::CopySmall => return bench_copy(&self.small),
            Case::CopyLarge => return bench_copy(&self.large),
            Case::CopyMarked => return bench_copy(&self.marked.terminal),
            Case::XscriptMarked => return bench_transcript(&self.marked.terminal),
            Case::XscriptWide => return bench_transcript(&self.wide.terminal),
            Case::PaintMarked => &self.marked,
            Case::PaintWide => &self.wide,
        };
        bench(WORK_ITERS, || {
            paint_once(&mut self.scene, &mut self.text, &self.theme, target);
        })
    }
}

fn main() {
    let mut ctx = Context::new();
    let mut best = [f64::INFINITY; CASES.len()];
    for round in 0..ROUNDS {
        for offset in 0..CASES.len() {
            let idx = (round + offset) % CASES.len();
            best[idx] = best[idx].min(ctx.run(CASES[idx]));
        }
    }
    for (&case, us) in CASES.iter().zip(best) {
        let pct = us / FRAME_BUDGET_US * 100.0;
        let name = case.name();
        println!("{name:<18} {us:>8.2} us/frame ({pct:>5.2} % of 16.7 ms)");
    }
}

fn scene_size(text: &TextSystem, terminal: &Terminal) -> (f32, f32) {
    let cell = text.cell_metrics();
    let screen = terminal.screen();
    (
        screen.cols() as f32 * cell.width,
        screen.rows() as f32 * cell.height,
    )
}

fn bench(iters: usize, mut run: impl FnMut()) -> f64 {
    let start = Instant::now();
    for _ in 0..iters {
        run();
    }
    start.elapsed().as_secs_f64() * MICROS_PER_SECOND / iters as f64
}

fn bench_copy(terminal: &Terminal) -> f64 {
    let screen = terminal.screen();
    bench(COPY_ITERS, || {
        drop(black_box(ScreenCopy::of(black_box(screen), 0.0)));
    })
}

fn bench_transcript(terminal: &Terminal) -> f64 {
    bench(WORK_ITERS, || {
        let copy = TranscriptCopy::of(black_box(terminal));
        drop(black_box(copy.encode(MAX_TRANSCRIPT_BYTES)));
    })
}

fn paint_once(scene: &mut Scene, text: &mut TextSystem, theme: &Theme, target: &PaintTarget) {
    let (w, h) = scene_size(text, &target.terminal);
    scene.reset(w, h);
    let paint = TerminalPaint {
        screen: &target.copy,
        origin: Point::new(0.0, 0.0),
        focused: false,
        scroll: 0.0,
        selected: &|_, _| false,
        links: &[],
    };
    draw_terminal(scene, text, theme, &paint);
    black_box(scene.quads());
}
