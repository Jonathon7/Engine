pub mod lifecycle;
pub mod pathfinding;
pub mod vehicle;
pub mod traffic;
pub mod platform;

pub struct Engine {
    running: bool,
    lifecycle: lifecycle::LifecycleManager
}

impl Engine {
    pub fn new(lifecycle_callbacks: lifecycle::LifecycleCallbacks) -> Self {
        Self {
            running:  false,
            lifecycle: lifecycle::LifecycleManager::new(lifecycle_callbacks)
        }
    }

    pub fn run(&mut self) {
        self.running = false;
        self.lifecycle.invoke_awake();
        self.lifecycle.invoke_start();

        while self.running {
            self.lifecycle.invoke_update();
        }
    }
}

