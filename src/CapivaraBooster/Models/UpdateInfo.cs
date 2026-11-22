namespace CapivaraBooster.Models;

public class UpdateInfo
{
    public string CurrentVersion { get; set; } = "1.0.0";
    public string LatestVersion { get; set; } = "1.0.0";
    public bool IsUpdateAvailable { get; set; }
    public string ReleaseTitle { get; set; } = string.Empty;
    public string ReleaseNotes { get; set; } = string.Empty;
    public string DownloadUrl { get; set; } = string.Empty;
    public string Sha256Hash { get; set; } = string.Empty;
    public DateTime PublishedAt { get; set; } = DateTime.Now;
}

public class VersionManifest
{
    public string Version { get; set; } = "1.0.0";
    public string ReleaseDate { get; set; } = string.Empty;
    public string Title { get; set; } = string.Empty;
    public string Changelog { get; set; } = string.Empty;
    public string InstallerUrl { get; set; } = string.Empty;
    public string Sha256 { get; set; } = string.Empty;
    public bool Mandatory { get; set; }
}
