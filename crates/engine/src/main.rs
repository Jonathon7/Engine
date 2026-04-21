use engine::Engine;
use engine::lifecycle::LifecycleCallbacks;
use std::sync::Mutex;
use libloading::{Library, Symbol};
use netcorehost::{nethost, pdcstr, pdcstring::PdCString};
use std::env;
use std::mem;

fn main() {
    let ffi_lib = unsafe {
        Library::new("engine_ffi.dll")
    }.expect("failed to load engine_ffi.dll");

    let get_callbacks: Symbol<unsafe extern "C" fn() -> *const Mutex<LifecycleCallbacks>> = unsafe {
        ffi_lib.get(b"get_lifecycle_callbacks")
    }.expect("missing get_lifecycle_callbacks");

    load_dotnet_runtime();

    let callbacks_ptr = unsafe { get_callbacks() };
    let callbacks_mutex = unsafe { &*callbacks_ptr };
    let mut guard = callbacks_mutex.lock().unwrap();
    let lifecycle_callbacks = LifecycleCallbacks {
        awake: mem::take(&mut guard.awake),
        start: mem::take(&mut guard.start),
        update: mem::take(&mut guard.update)
    };

    drop(guard);

    println!("awake count: {}", lifecycle_callbacks.awake.len());
    println!("start count: {}", lifecycle_callbacks.start.len());
    println!("update count: {}", lifecycle_callbacks.update.len());

    let mut engine = Engine::new(lifecycle_callbacks);
    engine.run();
}

fn load_dotnet_runtime()
{
    let exe_dir = env::current_exe()
        .expect("failed to get exe path")
        .parent()
        .unwrap()
        .to_path_buf();

    let config_path = exe_dir.join("Game.runtimeconfig.json");

    let config_pdcstring = PdCString::from_os_str(config_path.as_os_str())
        .expect("invalid path string");

    let hostfxr = nethost::load_hostfxr().unwrap();
    let context = hostfxr.initialize_for_runtime_config(config_pdcstring).unwrap();

    let assembly_path = exe_dir.join("Game.dll");

    let assembly_pdcstring = PdCString::from_os_str(assembly_path.as_os_str())
        .expect("invalid path string");

    let delegate_loader = context
        .get_delegate_loader_for_assembly(assembly_pdcstring)
        .unwrap();

    let initialize = delegate_loader.get_function_with_unmanaged_callers_only::<fn()>(
        pdcstr!("Game.Game, Game"),
        pdcstr!("Initialize")
    ).unwrap();

    initialize();
}
