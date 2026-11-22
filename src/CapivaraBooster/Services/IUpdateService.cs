using CapivaraBooster.Models;

namespace CapivaraBooster.Services;

public interface IUpdateService
{
    UpdateInfo? LatestUpdate { get; }
    bool IsChecking { get; }
    bool IsDownloading { get; }
    double DownloadProgress { get; }

    event Action<UpdateInfo>? UpdateAvailable;
    event Action<double>? DownloadProgressChanged;

    Task<UpdateInfo?> CheckForUpdatesAsync(bool silent = false);
    Task<bool> DownloadAndApplyUpdateAsync();
}
