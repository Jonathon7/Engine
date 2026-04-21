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

pub struct LifecycleManager {
    awake: Vec<extern "C" fn()>,
    start: Vec<extern "C" fn()>,
    update: Vec<extern "C" fn()>
}

impl LifecycleManager {
    pub fn new(lifecycle_callbacks: LifecycleCallbacks) -> Self {
        Self {
            awake: lifecycle_callbacks.awake,
            start: lifecycle_callbacks.start,
            update: lifecycle_callbacks.update
        }
    }

    pub fn invoke_awake(&self)
    {
        for cb in &self.awake {
            cb();
        }
    }

    pub fn invoke_start(&self)
    {
        for cb in &self.start {
            cb();
        }
    }

    pub fn invoke_update(&self)
    {
        for cb in &self.update {
            cb();
        }
    }
}
