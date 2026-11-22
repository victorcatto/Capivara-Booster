using System.IO;
using System.Text.Json;
using CapivaraBooster.Models;

namespace CapivaraBooster.Services;

public interface IRestoreService
{
    event Action<string>? LogMessage;

    Task<bool> CreateSystemRestorePointAsync(string description = "CapivaraBooster_PreTweak");
    Task<BackupRecord?> BackupFileAsync(string sourceFilePath, string category, string description);
    Task<bool> RestoreFileAsync(string backupId);
    IReadOnlyList<BackupRecord> GetBackups();
    string GetBackupFolder();
}

public class RestoreService : IRestoreService
{
    public event Action<string>? LogMessage;
    private readonly IProcessExecutor _executor;
    private readonly IRegistryService _registryService;
    private readonly List<BackupRecord> _backups = new();
    private readonly string _backupRoot;
    private readonly string _metadataFile;

    public RestoreService(IProcessExecutor executor, IRegistryService registryService)
    {
        _executor = executor;
        _registryService = registryService;

        _backupRoot = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData), "CapivaraBooster", "Backups");
        Directory.CreateDirectory(_backupRoot);
        _metadataFile = Path.Combine(_backupRoot, "backups_manifest.json");

        LoadManifest();
    }

    public string GetBackupFolder() => _backupRoot;

    public IReadOnlyList<BackupRecord> GetBackups() => _backups.AsReadOnly();

    public async Task<bool> CreateSystemRestorePointAsync(string description = "CapivaraBooster_PreTweak")
    {
        LogMessage?.Invoke("[Restore] Initiating Windows System Restore Point creation...");

        try
        {
            _registryService.SetDword(
                Microsoft.Win32.RegistryHive.LocalMachine,
                @"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore",
                "SystemRestorePointCreationFrequency",
                0,
                trackForRestore: false);

            string script = @"
$drive = $env:SystemDrive + '\'
try {
    Enable-ComputerRestore -Drive $drive -ErrorAction SilentlyContinue
} catch {}
Checkpoint-Computer -Description '" + description + @"' -RestorePointType 'MODIFY_SETTINGS' -ErrorAction Stop
";
            var result = await _executor.ExecutePowerShellAsync(script);
            if (result.IsSuccess)
            {
                LogMessage?.Invoke($"[Restore Success] System Restore Point created: '{description}'");
                return true;
            }
            else
            {
                LogMessage?.Invoke($"[Restore Warning] Could not create restore point: {result.StandardError}. Ensure System Protection is enabled.");
                return false;
            }
        }
        catch (Exception ex)
        {
            LogMessage?.Invoke($"[Restore Error] Failed to create restore point: {ex.Message}");
            return false;
        }
    }

    public async Task<BackupRecord?> BackupFileAsync(string sourceFilePath, string category, string description)
    {
        return await Task.Run(() =>
        {
            try
            {
                if (!File.Exists(sourceFilePath))
                {
                    LogMessage?.Invoke($"[Backup] Source file does not exist, skipping: {sourceFilePath}");
                    return null;
                }

                string fileName = Path.GetFileName(sourceFilePath);
                string timestamp = DateTime.Now.ToString("yyyyMMdd_HHmmss");
                string destFileName = $"{Path.GetFileNameWithoutExtension(fileName)}_{timestamp}{Path.GetExtension(fileName)}";
                string categoryDir = Path.Combine(_backupRoot, category);
                Directory.CreateDirectory(categoryDir);
                string destPath = Path.Combine(categoryDir, destFileName);

                File.Copy(sourceFilePath, destPath, overwrite: true);

                var record = new BackupRecord
                {
                    Name = fileName,
                    Category = category,
                    OriginalPath = sourceFilePath,
                    BackupPath = destPath,
                    Description = description,
                    CreatedAt = DateTime.Now
                };

                _backups.Insert(0, record);
                SaveManifest();

                LogMessage?.Invoke($"[Backup Created] Saved backup of '{fileName}' to '{destPath}'");
                return record;
            }
            catch (Exception ex)
            {
                LogMessage?.Invoke($"[Backup Error] Failed to backup '{sourceFilePath}': {ex.Message}");
                return null;
            }
        });
    }

    public async Task<bool> RestoreFileAsync(string backupId)
    {
        return await Task.Run(() =>
        {
            var record = _backups.FirstOrDefault(x => x.Id == backupId);
            if (record == null)
            {
                LogMessage?.Invoke($"[Restore Error] Backup ID {backupId} not found.");
                return false;
            }

            try
            {
                if (!File.Exists(record.BackupPath))
                {
                    LogMessage?.Invoke($"[Restore Error] Backup file missing at {record.BackupPath}");
                    return false;
                }

                string targetDir = Path.GetDirectoryName(record.OriginalPath)!;
                if (!Directory.Exists(targetDir))
                {
                    Directory.CreateDirectory(targetDir);
                }

                File.Copy(record.BackupPath, record.OriginalPath, overwrite: true);
                LogMessage?.Invoke($"[Restore Success] Restored '{record.Name}' to '{record.OriginalPath}'");
                return true;
            }
            catch (Exception ex)
            {
                LogMessage?.Invoke($"[Restore Error] Could not restore {record.Name}: {ex.Message}");
                return false;
            }
        });
    }

    private void SaveManifest()
    {
        try
        {
            string json = JsonSerializer.Serialize(_backups, new JsonSerializerOptions { WriteIndented = true });
            File.WriteAllText(_metadataFile, json);
        }
        catch { }
    }

    private void LoadManifest()
    {
        try
        {
            if (File.Exists(_metadataFile))
            {
                string json = File.ReadAllText(_metadataFile);
                var items = JsonSerializer.Deserialize<List<BackupRecord>>(json);
                if (items != null)
                {
                    _backups.Clear();
                    _backups.AddRange(items);
                }
            }
        }
        catch { }
    }
}
