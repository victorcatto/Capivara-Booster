namespace CapivaraBooster.Models;

public class BackupRecord
{
    public string Id { get; set; } = Guid.NewGuid().ToString("N");
    public DateTime CreatedAt { get; set; } = DateTime.Now;
    public string Name { get; set; } = string.Empty;
    public string Category { get; set; } = "Config";
    public string OriginalPath { get; set; } = string.Empty;
    public string BackupPath { get; set; } = string.Empty;
    public string Description { get; set; } = string.Empty;
}

public class RegistryValueBackup
{
    public string RegistryHive { get; set; } = string.Empty;
    public string SubKey { get; set; } = string.Empty;
    public string ValueName { get; set; } = string.Empty;
    public object? OriginalValue { get; set; }
    public string ValueKind { get; set; } = string.Empty;
    public bool ExistedBefore { get; set; }
    public DateTime Timestamp { get; set; } = DateTime.Now;
}
