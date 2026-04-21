use std::sync::{Mutex, LazyLock};

pub struct LifecycleCallbacks
{
    pub awake: Vec<extern "C" fn()>,
    pub start: Vec<extern "C" fn()>,
    pub update: Vec<extern "C" fn()>
}

impl LifecycleCallbacks {
    pub fn new() -> Self {
        Self {
            awake: Vec::new(),
            start: Vec::new(),
            update: Vec::new()
        }
    }
}

pub static LIFECYCLE_CALLBACKS: LazyLock<Mutex<LifecycleCallbacks>> = LazyLock::new(|| Mutex::new(LifecycleCallbacks::new()));

#[unsafe(no_mangle)]
pub extern "C" fn register_awake(callback: extern "C" fn())
{
    LIFECYCLE_CALLBACKS.lock().unwrap().awake.push(callback);
}

#[unsafe(no_mangle)]
pub extern "C" fn register_start(callback: extern "C" fn())
{
    LIFECYCLE_CALLBACKS.lock().unwrap().start.push(callback);
}

#[unsafe(no_mangle)]
pub extern "C" fn register_update(callback: extern "C" fn())
{
    LIFECYCLE_CALLBACKS.lock().unwrap().update.push(callback);
}

#[unsafe(no_mangle)]
pub extern "C" fn get_lifecycle_callbacks() -> *const Mutex<LifecycleCallbacks> {
    &*LIFECYCLE_CALLBACKS as *const _
}
