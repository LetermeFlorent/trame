const SPARE_FACTOR: usize = 4;
const QUIET_FRAMES: u32 = 120;

#[derive(Debug, Default)]
pub(super) struct Slack {
    quiet: u32,
}

impl Slack {
    pub(super) fn shrinks(&mut self, used: usize, capacity: usize, floor: usize) -> bool {
        if capacity <= floor || used.saturating_mul(SPARE_FACTOR) > capacity {
            self.quiet = 0;
            return false;
        }
        self.quiet += 1;
        if self.quiet < QUIET_FRAMES {
            return false;
        }
        self.quiet = 0;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{QUIET_FRAMES, Slack};

    #[test]
    fn shrinks_only_after_a_run_of_small_frames() {
        let mut slack = Slack::default();
        for _ in 1..QUIET_FRAMES {
            assert!(!slack.shrinks(10, 1024, 64));
        }
        assert!(slack.shrinks(10, 1024, 64));
        assert!(!slack.shrinks(10, 1024, 64));
    }

    #[test]
    fn a_busy_frame_starts_the_wait_again() {
        let mut slack = Slack::default();
        for _ in 1..QUIET_FRAMES {
            slack.shrinks(10, 1024, 64);
        }
        assert!(!slack.shrinks(600, 1024, 64));
        assert!(!slack.shrinks(10, 1024, 64));
    }

    #[test]
    fn never_goes_below_the_floor() {
        let mut slack = Slack::default();
        for _ in 0..QUIET_FRAMES * 2 {
            assert!(!slack.shrinks(0, 64, 64));
        }
    }
}
