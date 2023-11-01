use std::thread;
use std::thread::JoinHandle;

#[derive(Debug, Default)]
pub enum Job<T> {
    Done(T),
    InProgress(JoinHandle<T>),
    #[default]
    Empty,
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

    pub fn poll_owned(&mut self) -> Option<T> {
        let current_job = std::mem::take(self);
        match current_job {
            Self::Done(data) => Some(data),
            Self::InProgress(handle) => {
                if handle.is_finished() {
                    let value = handle.join().unwrap();
                    *self = Self::Done(value);
                } else {
                    *self = Self::InProgress(handle)
                }

                None
            }
            Self::Empty => None,
        }
    }

    pub fn poll(&mut self) -> Option<&T> {
        match self {
            Self::Done(val) => Some(val),
            Self::Empty => None,
            Self::InProgress(handle) if !handle.is_finished() => None,
            Self::InProgress(_) => {
                let current_job = std::mem::take(self);
                let Self::InProgress(handle) = current_job else {
                    unreachable!();
                };
                let value = handle.join().unwrap();
                *self = Self::Done(value);
                None
            }
        }
    }
}

// #[derive(Debug)]
// pub enum Job<T> {
//     Done(T),
//     InProgress(Option<JoinHandle<T>>),
//     Empty,
// }
//
// impl<T> Job<T> {
//     pub fn new<F>(f: F) -> Self
//     where
//         F: FnOnce() -> T,
//         F: Send + 'static,
//         T: Send + 'static,
//     {
//         Self::InProgress(Some(thread::spawn(f)))
//     }
//
//     pub fn poll_owned(&mut self) -> Option<T> {
//         match self {
//             Self::Done(_) => unreachable!(),
//             Self::Empty => None,
//             Self::InProgress(Some(handle)) if !handle.is_finished() => None,
//             Self::InProgress(None) => None,
//             Self::InProgress(handle) => {
//                 let val = handle.take()?.join().ok()?;
//                 *self = Self::Empty;
//                 Some(val)
//             }
//         }
//     }
//
//     pub fn poll(&mut self) -> Option<&T> {
//         match self {
//             Self::Done(val) => Some(val),
//             Self::Empty => None,
//             Self::InProgress(Some(handle)) if !handle.is_finished() => None,
//             Self::InProgress(None) => None,
//             Self::InProgress(handle) => {
//                 let val = handle.take()?.join().ok()?;
//                 *self = Self::Done(val);
//                 None
//             }
//         }
//     }
// }
