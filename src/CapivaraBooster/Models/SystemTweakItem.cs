using CapivaraBooster.Common;

namespace CapivaraBooster.Models;

public enum TweakCategory
{
    Latency,
    Power,
    Gaming,
    Network,
    Privacy
}

public class SystemTweakItem : ObservableObject
{
    private bool _isEnabled;
    private bool _isProcessing;

    public string Key { get; init; } = string.Empty;
    public string Title { get; init; } = string.Empty;
    public string Description { get; init; } = string.Empty;
    public TweakCategory Category { get; init; } = TweakCategory.Latency;
    public bool RequiresReboot { get; init; } = false;

    public bool IsEnabled
    {
        get => _isEnabled;
        set => SetProperty(ref _isEnabled, value);
    }

    public bool IsProcessing
    {
        get => _isProcessing;
        set => SetProperty(ref _isProcessing, value);
    }
}
