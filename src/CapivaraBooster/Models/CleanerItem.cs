using CapivaraBooster.Common;

namespace CapivaraBooster.Models;

public enum CleanerCategory
{
    WindowsTemp,
    SystemTemp,
    Prefetch,
    SoftwareDistribution,
    RecycleBin,
    DnsCache,
    DirectXShader,
    NvidiaShader,
    AmdShader
}

public class CleanerItem : ObservableObject
{
    private bool _isSelected = true;
    private long _sizeBytes;
    private bool _isScanning;
    private bool _isCleaned;

    public CleanerCategory Category { get; init; }
    public string DisplayName { get; init; } = string.Empty;
    public string Description { get; init; } = string.Empty;
    public string TargetPath { get; init; } = string.Empty;

    public bool IsSelected
    {
        get => _isSelected;
        set => SetProperty(ref _isSelected, value);
    }

    public long SizeBytes
    {
        get => _sizeBytes;
        set
        {
            if (SetProperty(ref _sizeBytes, value))
            {
                OnPropertyChanged(nameof(FormattedSize));
            }
        }
    }

    public bool IsScanning
    {
        get => _isScanning;
        set => SetProperty(ref _isScanning, value);
    }

    public bool IsCleaned
    {
        get => _isCleaned;
        set => SetProperty(ref _isCleaned, value);
    }

    public string FormattedSize
    {
        get
        {
            if (Category == CleanerCategory.DnsCache)
            {
                return "DNS Records";
            }

            if (SizeBytes < 1024)
                return $"{SizeBytes} B";
            if (SizeBytes < 1024 * 1024)
                return $"{SizeBytes / 1024.0:F1} KB";
            if (SizeBytes < 1024 * 1024 * 1024)
                return $"{SizeBytes / (1024.0 * 1024.0):F1} MB";

            return $"{SizeBytes / (1024.0 * 1024.0 * 1024.0):F2} GB";
        }
    }
}
