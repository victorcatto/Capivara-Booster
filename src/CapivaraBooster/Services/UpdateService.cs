using System.Diagnostics;
using System.IO;
using System.Net.Http;
using System.Reflection;
using System.Security.Cryptography;
using System.Text.Json;
using System.Windows;
using CapivaraBooster.Models;

namespace CapivaraBooster.Services;

public class UpdateService : IUpdateService
{
    private const string RepoOwner = "victorcatto93";
    private const string RepoName = "Capivara-Booster";
    private const string VersionManifestUrl = $"https://raw.githubusercontent.com/{RepoOwner}/{RepoName}/main/version.json";
    private const string GitHubApiReleasesUrl = $"https://api.github.com/repos/{RepoOwner}/{RepoName}/releases/latest";

    private readonly HttpClient _http;
    private readonly ILocalizationService _loc;

    private UpdateInfo? _latestUpdate;
    private bool _isChecking;
    private bool _isDownloading;
    private double _downloadProgress;

    public UpdateInfo? LatestUpdate => _latestUpdate;
    public bool IsChecking => _isChecking;
    public bool IsDownloading => _isDownloading;
    public double DownloadProgress => _downloadProgress;

    public event Action<UpdateInfo>? UpdateAvailable;
    public event Action<double>? DownloadProgressChanged;

    public UpdateService(ILocalizationService loc)
    {
        _loc = loc;
        _http = new HttpClient();
        _http.DefaultRequestHeaders.UserAgent.ParseAdd("CapivaraBooster-Client/1.0.0");
    }

    public async Task<UpdateInfo?> CheckForUpdatesAsync(bool silent = false)
    {
        if (_isChecking) return null;
        _isChecking = true;

        try
        {
            string currentVerStr = Assembly.GetExecutingAssembly().GetName().Version?.ToString(3) ?? "1.0.0";
            var currentVersion = ParseSemanticVersion(currentVerStr);

            // 1. First attempt: Raw version.json manifest
            try
            {
                var manifestJson = await _http.GetStringAsync(VersionManifestUrl);
                var manifest = JsonSerializer.Deserialize<VersionManifest>(manifestJson, new JsonSerializerOptions { PropertyNameCaseInsensitive = true });
                if (manifest != null)
                {
                    var remoteVersion = ParseSemanticVersion(manifest.Version);
                    if (remoteVersion > currentVersion)
                    {
                        _latestUpdate = new UpdateInfo
                        {
                            CurrentVersion = currentVerStr,
                            LatestVersion = manifest.Version,
                            IsUpdateAvailable = true,
                            ReleaseTitle = manifest.Title,
                            ReleaseNotes = manifest.Changelog,
                            DownloadUrl = manifest.InstallerUrl,
                            Sha256Hash = manifest.Sha256,
                            PublishedAt = DateTime.Now
                        };

                        UpdateAvailable?.Invoke(_latestUpdate);
                        return _latestUpdate;
                    }
                }
            }
            catch { /* fallback to GitHub Releases API */ }

            // 2. Fallback: Query GitHub Releases API
            try
            {
                var response = await _http.GetStringAsync(GitHubApiReleasesUrl);
                using var doc = JsonDocument.Parse(response);
                var root = doc.RootElement;

                string tagName = root.GetProperty("tag_name").GetString() ?? "";
                string cleanTag = tagName.TrimStart('v', 'V');
                var remoteVer = ParseSemanticVersion(cleanTag);

                if (remoteVer > currentVersion)
                {
                    string title = root.TryGetProperty("name", out var np) ? np.GetString() ?? "" : tagName;
                    string notes = root.TryGetProperty("body", out var bp) ? bp.GetString() ?? "" : "";
                    string downloadUrl = "";

                    if (root.TryGetProperty("assets", out var assets) && assets.ValueKind == JsonValueKind.Array)
                    {
                        foreach (var asset in assets.EnumerateArray())
                        {
                            string assetName = asset.GetProperty("name").GetString() ?? "";
                            if (assetName.EndsWith(".exe", StringComparison.OrdinalIgnoreCase) || assetName.EndsWith(".msi", StringComparison.OrdinalIgnoreCase))
                            {
                                downloadUrl = asset.GetProperty("browser_download_url").GetString() ?? "";
                                break;
                            }
                        }
                    }

                    _latestUpdate = new UpdateInfo
                    {
                        CurrentVersion = currentVerStr,
                        LatestVersion = cleanTag,
                        IsUpdateAvailable = true,
                        ReleaseTitle = title,
                        ReleaseNotes = notes,
                        DownloadUrl = downloadUrl,
                        PublishedAt = DateTime.Now
                    };

                    UpdateAvailable?.Invoke(_latestUpdate);
                    return _latestUpdate;
                }
            }
            catch { /* offline or repo not created yet */ }

            _latestUpdate = new UpdateInfo
            {
                CurrentVersion = currentVerStr,
                LatestVersion = currentVerStr,
                IsUpdateAvailable = false
            };

            return _latestUpdate;
        }
        finally
        {
            _isChecking = false;
        }
    }

    public async Task<bool> DownloadAndApplyUpdateAsync()
    {
        if (_latestUpdate == null || string.IsNullOrEmpty(_latestUpdate.DownloadUrl))
        {
            return false;
        }

        _isDownloading = true;
        _downloadProgress = 0;
        DownloadProgressChanged?.Invoke(0);

        string tempInstallerPath = Path.Combine(Path.GetTempPath(), $"CapivaraBooster-Setup-v{_latestUpdate.LatestVersion}.exe");

        try
        {
            using (var response = await _http.GetAsync(_latestUpdate.DownloadUrl, HttpCompletionOption.ResponseHeadersRead))
            {
                response.EnsureSuccessStatusCode();

                long? totalBytes = response.Content.Headers.ContentLength;
                using var stream = await response.Content.ReadAsStreamAsync();
                using var fileStream = new FileStream(tempInstallerPath, FileMode.Create, FileAccess.Write, FileShare.None, 8192, true);

                var buffer = new byte[8192];
                long totalRead = 0;
                int bytesRead;

                while ((bytesRead = await stream.ReadAsync(buffer, 0, buffer.Length)) > 0)
                {
                    await fileStream.WriteAsync(buffer, 0, bytesRead);
                    totalRead += bytesRead;

                    if (totalBytes.HasValue && totalBytes.Value > 0)
                    {
                        _downloadProgress = (double)totalRead / totalBytes.Value * 100.0;
                        DownloadProgressChanged?.Invoke(_downloadProgress);
                    }
                }
            }

            // Verify integrity if hash provided
            if (!string.IsNullOrEmpty(_latestUpdate.Sha256Hash))
            {
                using var sha = SHA256.Create();
                using var fs = File.OpenRead(tempInstallerPath);
                byte[] hash = await sha.ComputeHashAsync(fs);
                string computed = Convert.ToHexString(hash);

                if (!string.Equals(computed, _latestUpdate.Sha256Hash, StringComparison.OrdinalIgnoreCase))
                {
                    MessageBox.Show(
                        _loc.IsPortuguese ? "Falha na verificação de integridade do instalador baixado." : "Downloaded installer hash verification failed.",
                        "Capivara Booster Update",
                        MessageBoxButton.OK,
                        MessageBoxImage.Error);
                    return false;
                }
            }

            // Launch installer with silent or UI parameters and close current instance
            var psi = new ProcessStartInfo
            {
                FileName = tempInstallerPath,
                Arguments = "/SILENT /CLOSEAPPLICATIONS /RESTARTAPPLICATIONS",
                UseShellExecute = true,
                Verb = "runas"
            };

            Process.Start(psi);

            Application.Current.Dispatcher.Invoke(() =>
            {
                Application.Current.Shutdown();
            });

            return true;
        }
        catch (Exception ex)
        {
            MessageBox.Show(
                (_loc.IsPortuguese ? "Erro ao atualizar: " : "Update error: ") + ex.Message,
                "Capivara Booster",
                MessageBoxButton.OK,
                MessageBoxImage.Warning);
            return false;
        }
        finally
        {
            _isDownloading = false;
        }
    }

    private Version ParseSemanticVersion(string ver)
    {
        string clean = ver.Trim().TrimStart('v', 'V');
        var parts = clean.Split('.');
        int major = parts.Length > 0 && int.TryParse(parts[0], out int mj) ? mj : 1;
        int minor = parts.Length > 1 && int.TryParse(parts[1], out int mn) ? mn : 0;
        int build = parts.Length > 2 && int.TryParse(parts[2], out int bd) ? bd : 0;
        return new Version(major, minor, build);
    }
}
