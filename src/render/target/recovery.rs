pub(super) enum Outcome<T> {
    Ready(T),
    Suboptimal(T),
    Outdated,
    Lost,
    Timeout,
    Occluded,
    OutOfMemory,
    Validation,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StepStatus {
    Ok,
    OutOfMemory,
    Failed,
}

pub(super) enum FrameResult<T> {
    Ready(T),
    Skip,
    DeviceLost,
}

pub(super) trait SurfaceOps {
    type Texture;
    fn has_surface(&self) -> bool;
    fn acquire_next(&mut self) -> Outcome<Self::Texture>;
    fn reconfigure(&mut self) -> StepStatus;
    fn recreate(&mut self) -> StepStatus;
    fn device_lost(&self) -> bool;
}

pub(super) fn acquire_step<O: SurfaceOps>(
    ops: &mut O,
    pending_configure: bool,
    log: &mut dyn FnMut(&str),
) -> (FrameResult<O::Texture>, bool) {
    if ops.device_lost() {
        return (FrameResult::DeviceLost, false);
    }
    if !ops.has_surface() {
        return (recover_surface(ops), false);
    }
    if pending_configure {
        match ops.reconfigure() {
            StepStatus::Ok => {}
            StepStatus::OutOfMemory => return (recreate_on_oom(ops, log), false),
            StepStatus::Failed => {
                let result = skip_or_lost(ops);
                let restore = matches!(result, FrameResult::Skip);
                return (result, restore);
            }
        }
    }
    let result = match ops.acquire_next() {
        Outcome::Ready(texture) => FrameResult::Ready(texture),
        Outcome::Suboptimal(texture) => {
            drop(texture);
            retry_after_reconfigure(ops, log)
        }
        Outcome::Outdated => retry_after_reconfigure(ops, log),
        Outcome::Timeout => handle_timeout(ops, log),
        Outcome::Lost => handle_lost(ops, log, "the window surface was lost, recreating"),
        Outcome::OutOfMemory => recreate_on_oom(ops, log),
        Outcome::Validation => {
            handle_lost(ops, log, "the window surface validation failed, recreating")
        }
        Outcome::Occluded => FrameResult::Skip,
    };
    (result, false)
}

fn handle_timeout<O: SurfaceOps>(
    ops: &mut O,
    log: &mut dyn FnMut(&str),
) -> FrameResult<O::Texture> {
    log("the window surface timed out, reconfiguring");
    match ops.reconfigure() {
        StepStatus::OutOfMemory => recreate_on_oom(ops, log),
        _ => skip_or_lost(ops),
    }
}

fn handle_lost<O: SurfaceOps>(
    ops: &mut O,
    log: &mut dyn FnMut(&str),
    message: &str,
) -> FrameResult<O::Texture> {
    if ops.device_lost() {
        return FrameResult::DeviceLost;
    }
    log(message);
    recover_surface(ops)
}

fn retry_after_reconfigure<O: SurfaceOps>(
    ops: &mut O,
    log: &mut dyn FnMut(&str),
) -> FrameResult<O::Texture> {
    match ops.reconfigure() {
        StepStatus::Ok => match ops.acquire_next() {
            Outcome::Ready(texture) | Outcome::Suboptimal(texture) => FrameResult::Ready(texture),
            Outcome::Lost | Outcome::OutOfMemory | Outcome::Validation => handle_lost(
                ops,
                log,
                "the window surface could not reconfigure, recreating",
            ),
            Outcome::Timeout => {
                log("the window surface timed out after reconfigure");
                skip_or_lost(ops)
            }
            Outcome::Outdated | Outcome::Occluded => skip_or_lost(ops),
        },
        StepStatus::OutOfMemory => recreate_on_oom(ops, log),
        StepStatus::Failed => skip_or_lost(ops),
    }
}

fn recreate_on_oom<O: SurfaceOps>(
    ops: &mut O,
    log: &mut dyn FnMut(&str),
) -> FrameResult<O::Texture> {
    log("the window surface ran out of memory, recreating");
    recover_surface(ops)
}

fn recover_surface<O: SurfaceOps>(ops: &mut O) -> FrameResult<O::Texture> {
    if ops.device_lost() || ops.recreate() != StepStatus::Ok {
        return FrameResult::DeviceLost;
    }
    match ops.acquire_next() {
        Outcome::Ready(texture) | Outcome::Suboptimal(texture) => FrameResult::Ready(texture),
        Outcome::Lost | Outcome::OutOfMemory | Outcome::Validation => FrameResult::DeviceLost,
        Outcome::Timeout | Outcome::Outdated | Outcome::Occluded => skip_or_lost(ops),
    }
}

fn skip_or_lost<O: SurfaceOps>(ops: &O) -> FrameResult<O::Texture> {
    if ops.device_lost() {
        FrameResult::DeviceLost
    } else {
        FrameResult::Skip
    }
}
