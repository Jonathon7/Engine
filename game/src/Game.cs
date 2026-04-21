using System.Reflection;
using System.Runtime.InteropServices;

namespace Game;

public static class Game
{
    private static readonly List<Delegate> _behaviors = [];

    [UnmanagedCallersOnly]
    public static void Initialize()
    {
        var assemblies = AppDomain.CurrentDomain.GetAssemblies();

        List<Type> behaviors = [];
        foreach (var assembly in assemblies)
        {
            var types = assembly.GetTypes().Where(t => t.IsSubclassOf(typeof(Behavior)));
            behaviors.AddRange(types);
        }

        foreach (var behavior in behaviors)
        {
            var instance = Activator.CreateInstance(behavior);
            var awakeDel = instance?.GetType().GetMethod("Awake", BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic)?
                .CreateDelegate<Action>(instance);
            var startDel = instance?.GetType().GetMethod("Start", BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic)?
                .CreateDelegate<Action>(instance);
            var updateDel = instance?.GetType().GetMethod("Update", BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic)?
                .CreateDelegate<Action>(instance);

            if (awakeDel != null)
            {
                _behaviors.Add(awakeDel);
                register_awake(Marshal.GetFunctionPointerForDelegate(awakeDel));
            }

            if (startDel != null)
            {
                _behaviors.Add(startDel);
                register_start(Marshal.GetFunctionPointerForDelegate(startDel));
            }

            if (updateDel != null)
            {
                _behaviors.Add(updateDel);
                register_update(Marshal.GetFunctionPointerForDelegate(updateDel));
            }
        }
    }

    [DllImport("engine_ffi")]
    private static extern void register_awake(IntPtr callback);

    [DllImport("engine_ffi")]
    private static extern void register_start(IntPtr callback);
    
    [DllImport("engine_ffi")]
    private static extern void register_update(IntPtr callback);
}

public abstract class Behavior
{}
