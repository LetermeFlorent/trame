use std::thread::sleep;
use std::time::Duration;

use super::*;

const REPAINT_LATER: Duration = Duration::from_millis(1);

#[test]
fn an_idle_ai_repainting_its_status_line_lets_its_hidden_tab_and_window_rest() {
    let mut power = Power::new([true, true]);
    power.set_tabs([tab(1, false)]);
    power.set_background(true);
    power.output(SessionId(1), false);
    let rest_at = Instant::now() + REST_AFTER;
    sleep(REPAINT_LATER);
    power.output(SessionId(1), true);
    assert!(power.deadline().is_some_and(|at| at <= rest_at));
    power.tick(rest_at);
    assert_eq!(power.mode(), Mode::Family);
    assert_eq!(power.take_rested(), vec![SessionId(1)]);
}
