use std::thread;
use std::thread::JoinHandle;

#[derive(Debug, Default)]
pub enum Job<T> {
    InProgress(JoinHandle<T>),

    #[default]
    Finished,
}

impl<T> Job<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        Self::InProgress(thread::spawn(f))
    }

    pub fn poll(&mut self) -> Option<T> {
        match self {
            Self::Finished => None,
            Self::InProgress(handle) if !handle.is_finished() => None,
            Self::InProgress(_) => {
                let value = std::mem::take(self);
                match value {
                    Job::InProgress(handle) => Some(handle.join().unwrap()),
                    Job::Finished => unreachable!(),
                }
            }
        }
    }
}
