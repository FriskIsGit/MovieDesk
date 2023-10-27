use std::thread;
use std::thread::JoinHandle;

pub struct Job<T> {
    handle: Option<JoinHandle<T>>
}
impl<T> Job<T>{
    pub fn empty() -> Self {
        Self{
            handle: None
        }
    }
    pub fn set(&mut self, handle: JoinHandle<T>) {
        let _ = std::mem::replace(&mut self.handle, Some(handle)); // mem::swap?
    }
    pub fn is_any_and_finished(&self) -> bool {
        //here we can only avoid calling as_ref or use Deref?
        if self.handle.is_none() {
            return false;
        }
        self.handle.as_ref().unwrap().is_finished()
    }
    //permanently moves result, use with: if job.is_some_and_finished()
    pub fn take_result(&mut self) -> thread::Result<T> {
        let handle = std::mem::replace(&mut self.handle, None);
        handle.unwrap().join()
    }

    //any == some, if it's active/finished
    pub fn is_any(&self) -> bool {
        self.handle.is_some()
    }
    pub fn is_empty(&self) -> bool {
        self.handle.is_none()
    }
}
