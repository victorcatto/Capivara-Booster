using CapivaraBooster.Common;

namespace CapivaraBooster.Models;

public class GameOptimizationState : ObservableObject
{
    private bool _isDetected;
    private string _installedPath = string.Empty;
    private bool _highPriorityApplied;
    private bool _disableCore0Applied;
    private bool _disableFseApplied;
    private bool _highDpiApplied;
    private bool _gameSpecificApplied;
    private bool _isBusy;

    public string GameId { get; init; } = string.Empty;
    public string DisplayName { get; init; } = string.Empty;
    public string ExeName { get; init; } = string.Empty;
    public string IconColorHex { get; init; } = "#00E5FF";

    public bool IsDetected
    {
        get => _isDetected;
        set => SetProperty(ref _isDetected, value);
    }

    public string InstalledPath
    {
        get => _installedPath;
        set => SetProperty(ref _installedPath, value);
    }

    public bool HighPriorityApplied
    {
        get => _highPriorityApplied;
        set => SetProperty(ref _highPriorityApplied, value);
    }

    public bool DisableCore0Applied
    {
        get => _disableCore0Applied;
        set => SetProperty(ref _disableCore0Applied, value);
    }

    public bool DisableFseApplied
    {
        get => _disableFseApplied;
        set => SetProperty(ref _disableFseApplied, value);
    }

    public bool HighDpiApplied
    {
        get => _highDpiApplied;
        set => SetProperty(ref _highDpiApplied, value);
    }

    public bool GameSpecificApplied
    {
        get => _gameSpecificApplied;
        set => SetProperty(ref _gameSpecificApplied, value);
    }

    public bool IsBusy
    {
        get => _isBusy;
        set => SetProperty(ref _isBusy, value);
    }
}
