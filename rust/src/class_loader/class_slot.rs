use crate::{
    class_loader::{load_error::LoadError, ms_api::MSBox},
    oops::klass::Klass,
};

#[derive(Debug)]
pub enum ClassLoadState {
    Loading { owner: std::thread::ThreadId },
    Loaded(MSBox<Klass>),
    Failed(LoadError),
}

#[derive(Debug)]
pub struct ClassSlot {
    pub state: parking_lot::Mutex<ClassLoadState>,
    pub completed: parking_lot::Condvar,
}

impl Default for ClassSlot {
    fn default() -> Self {
        Self {
            state: parking_lot::Mutex::new(ClassLoadState::Loading {
                owner: std::thread::current().id(),
            }),
            completed: parking_lot::Condvar::new(),
        }
    }
}
