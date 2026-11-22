using CapivaraBooster.Common;

namespace CapivaraBooster.Models;

public class BloatwareItem : ObservableObject
{
    private bool _isSelected;
    private bool _isInstalled = true;
    private bool _isRemoved;

    public string Id { get; init; } = string.Empty;
    public string Name { get; init; } = string.Empty;
    public string PackagePattern { get; init; } = string.Empty;
    public string Description { get; init; } = string.Empty;
    public string Category { get; init; } = "General";
    public bool IsSafe { get; init; } = true;

    public bool IsSelected
    {
        get => _isSelected;
        set => SetProperty(ref _isSelected, value);
    }

    public bool IsInstalled
    {
        get => _isInstalled;
        set => SetProperty(ref _isInstalled, value);
    }

    public bool IsRemoved
    {
        get => _isRemoved;
        set => SetProperty(ref _isRemoved, value);
    }
}
