/// A cooperative scheduler for concurrent LuwiScript tasks.
///
/// Currently a placeholder — the real scheduler will manage multiple
/// VM instances (coroutines) on a single OS thread, yielding and
/// resuming based on I/O readiness and explicit `spawn`/`await`.
pub struct Scheduler;

impl Scheduler {
    pub fn new() -> Self {
        Scheduler
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
