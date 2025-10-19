use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug)]
pub struct ReleaseTimeTracker {
    release_times: VecDeque<Duration>,
    max_size: usize,
}

impl ReleaseTimeTracker {
    pub fn new(max_size: usize) -> ReleaseTimeTracker {
        ReleaseTimeTracker {
            release_times: VecDeque::new(),
            max_size,
        }
    }
    pub fn add_release_time(&mut self, duration: Duration) {
        self.release_times.push_back(duration);
        if self.release_times.len() > self.max_size {
            self.release_times.pop_front();
        }
    }
    pub fn average_release_time(&self) -> Duration {
        if self.release_times.is_empty() {
            return Duration::from_millis(0);
        }
        let sum: Duration = self.release_times.iter().sum();
        sum / self.release_times.len() as u32
    }
}

#[derive(Debug)]
pub struct GameStatistics {
    notes_count: u32,
    last_save_count: u32,
    release_tracker: ReleaseTimeTracker,
}

impl GameStatistics {
    pub fn new() -> GameStatistics {
        GameStatistics {
            notes_count: 0,
            last_save_count: 0,
            release_tracker: ReleaseTimeTracker::new(2000),
        }
    }

    pub fn increment_notes(&mut self) {
        self.notes_count += 1;
    }

    pub fn notes_count(&self) -> u32 {
        self.notes_count
    }

    pub fn set_last_saved_count(&mut self) {
        self.last_save_count = self.notes_count;
    }

    pub fn difference_notes_count(&self) -> u32 {
        self.notes_count - self.last_save_count
    }

    pub fn add_release_time(&mut self, duration: Duration) {
        self.release_tracker.add_release_time(duration)
    }

    pub fn average_release_time(&self) -> Duration {
        self.release_tracker.average_release_time()
    }
}
