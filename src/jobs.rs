use std::thread;
use std::thread::JoinHandle;

pub struct Job<T> {
    handle: Option<JoinHandle<T>>,
}

impl<T> Job<T> {
    pub fn empty() -> Self {
        Self { handle: None }
    }

    pub fn set(&mut self, handle: JoinHandle<T>) {
        self.handle = Some(handle);
    }

    pub fn poll(&mut self) -> Option<T> {
        let is_finished = self.handle.as_ref().map(|h| h.is_finished()).unwrap_or(false);

        match is_finished {
            true => self.handle.take().and_then(|h| h.join().ok()),
            false => None,
        }
    }
}

#[derive(Debug)]
pub enum Job2<T> {
    Done(T),
    InFlight(Option<JoinHandle<T>>),
    Empty,
}

impl<T> Job2<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        Self::InFlight(Some(thread::spawn(f)))
    }

    pub fn poll_owned(&mut self) -> Option<T> {
        match self {
            Self::Done(_) => unreachable!(),
            Self::Empty => None,
            Self::InFlight(Some(handle)) if !handle.is_finished() => None,
            Self::InFlight(None) => None,
            Self::InFlight(handle) => {
                let val = handle.take()?.join().ok()?;
                *self = Self::Empty;
                Some(val)
            }
        }
    }

    pub fn poll(&mut self) -> Option<&T> {
        match self {
            Self::Done(val) => Some(val),
            Self::Empty => None,
            Self::InFlight(Some(handle)) if !handle.is_finished() => None,
            Self::InFlight(None) => None,
            Self::InFlight(handle) => {
                let val = handle.take()?.join().ok()?;
                *self = Self::Done(val);
                let Self::Done(val) = self else { unreachable!() };
                Some(val)
            }
        }
    }
}
