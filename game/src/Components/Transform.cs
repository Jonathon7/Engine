namespace Game;

public sealed class Transform : Behavior
{
    private void Awake()
    {
        Console.WriteLine($"{nameof(Transform)} Awake");
    }

    private void Start()
    {
        Console.WriteLine($"{nameof(Transform)} Start");
    }

    private void Update()
    {
        Console.WriteLine($"{nameof(Transform)} Update");
    }
}
