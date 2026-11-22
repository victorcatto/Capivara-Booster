using System.IO;
using System.Security;
using System.Text.Json;
using CapivaraBooster.Models;
using Microsoft.Win32;

namespace CapivaraBooster.Services;

public class RegistryService : IRegistryService
{
    public event Action<string>? LogMessage;
    private readonly List<RegistryValueBackup> _trackedChanges = new();
    private readonly string _backupStorePath;

    public RegistryService()
    {
        string appData = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData), "CapivaraBooster");
        Directory.CreateDirectory(appData);
        _backupStorePath = Path.Combine(appData, "registry_rollback_store.json");
        LoadTrackedChanges();
    }

    private void Log(string msg) => LogMessage?.Invoke(msg);

    public bool SetDword(RegistryHive hive, string subKeyPath, string valueName, int value, bool trackForRestore = true)
    {
        try
        {
            if (trackForRestore) TrackCurrentValue(hive, subKeyPath, valueName);

            using var baseKey = RegistryKey.OpenBaseKey(hive, RegistryView.Registry64);
            using var subKey = baseKey.CreateSubKey(subKeyPath, RegistryKeyPermissionCheck.ReadWriteSubTree);
            subKey.SetValue(valueName, value, RegistryValueKind.DWord);
            Log($"[Registry] Set {hive}\\{subKeyPath}\\{valueName} = {value} (DWORD)");
            return true;
        }
        catch (SecurityException ex)
        {
            Log($"[Registry Access Denied] Failed to write {hive}\\{subKeyPath}\\{valueName}: {ex.Message}");
            return false;
        }
        catch (Exception ex)
        {
            Log($"[Registry Error] Failed to write {hive}\\{subKeyPath}\\{valueName}: {ex.Message}");
            return false;
        }
    }

    public bool SetString(RegistryHive hive, string subKeyPath, string valueName, string value, bool trackForRestore = true)
    {
        try
        {
            if (trackForRestore) TrackCurrentValue(hive, subKeyPath, valueName);

            using var baseKey = RegistryKey.OpenBaseKey(hive, RegistryView.Registry64);
            using var subKey = baseKey.CreateSubKey(subKeyPath, RegistryKeyPermissionCheck.ReadWriteSubTree);
            subKey.SetValue(valueName, value, RegistryValueKind.String);
            Log($"[Registry] Set {hive}\\{subKeyPath}\\{valueName} = \"{value}\" (String)");
            return true;
        }
        catch (Exception ex)
        {
            Log($"[Registry Error] Failed to write {hive}\\{subKeyPath}\\{valueName}: {ex.Message}");
            return false;
        }
    }

    public bool DeleteValue(RegistryHive hive, string subKeyPath, string valueName, bool trackForRestore = true)
    {
        try
        {
            if (trackForRestore) TrackCurrentValue(hive, subKeyPath, valueName);

            using var baseKey = RegistryKey.OpenBaseKey(hive, RegistryView.Registry64);
            using var subKey = baseKey.OpenSubKey(subKeyPath, true);
            if (subKey != null && subKey.GetValue(valueName) != null)
            {
                subKey.DeleteValue(valueName, false);
                Log($"[Registry] Deleted value {hive}\\{subKeyPath}\\{valueName}");
                return true;
            }
            return false;
        }
        catch (Exception ex)
        {
            Log($"[Registry Error] Failed to delete value {hive}\\{subKeyPath}\\{valueName}: {ex.Message}");
            return false;
        }
    }

    public bool DeleteKeyTree(RegistryHive hive, string subKeyPath)
    {
        try
        {
            using var baseKey = RegistryKey.OpenBaseKey(hive, RegistryView.Registry64);
            baseKey.DeleteSubKeyTree(subKeyPath, false);
            Log($"[Registry] Deleted subkey tree {hive}\\{subKeyPath}");
            return true;
        }
        catch (Exception ex)
        {
            Log($"[Registry Error] Failed to delete key {hive}\\{subKeyPath}: {ex.Message}");
            return false;
        }
    }

    public object? GetValue(RegistryHive hive, string subKeyPath, string valueName)
    {
        try
        {
            using var baseKey = RegistryKey.OpenBaseKey(hive, RegistryView.Registry64);
            using var subKey = baseKey.OpenSubKey(subKeyPath, false);
            return subKey?.GetValue(valueName);
        }
        catch
        {
            return null;
        }
    }

    public int? GetDword(RegistryHive hive, string subKeyPath, string valueName)
    {
        var val = GetValue(hive, subKeyPath, valueName);
        return val is int i ? i : null;
    }

    public string? GetString(RegistryHive hive, string subKeyPath, string valueName)
    {
        var val = GetValue(hive, subKeyPath, valueName);
        return val?.ToString();
    }

    public bool KeyExists(RegistryHive hive, string subKeyPath)
    {
        try
        {
            using var baseKey = RegistryKey.OpenBaseKey(hive, RegistryView.Registry64);
            using var subKey = baseKey.OpenSubKey(subKeyPath, false);
            return subKey != null;
        }
        catch
        {
            return false;
        }
    }

    public IReadOnlyList<RegistryValueBackup> GetTrackedBackups() => _trackedChanges.AsReadOnly();

    public async Task<bool> RevertAllTrackedAsync()
    {
        return await Task.Run(() =>
        {
            bool success = true;
            var list = _trackedChanges.ToList();
            Log($"[Registry Rollback] Reverting {list.Count} tracked registry keys...");

            for (int i = list.Count - 1; i >= 0; i--)
            {
                var entry = list[i];
                try
                {
                    var hive = Enum.Parse<RegistryHive>(entry.RegistryHive);
                    using var baseKey = RegistryKey.OpenBaseKey(hive, RegistryView.Registry64);

                    if (!entry.ExistedBefore)
                    {
                        using var subKey = baseKey.OpenSubKey(entry.SubKey, true);
                        if (subKey != null && subKey.GetValue(entry.ValueName) != null)
                        {
                            subKey.DeleteValue(entry.ValueName, false);
                            Log($"[Rollback] Removed added value: {hive}\\{entry.SubKey}\\{entry.ValueName}");
                        }
                    }
                    else
                    {
                        using var subKey = baseKey.CreateSubKey(entry.SubKey, RegistryKeyPermissionCheck.ReadWriteSubTree);
                        if (Enum.TryParse<RegistryValueKind>(entry.ValueKind, out var kind))
                        {
                            if (kind == RegistryValueKind.DWord && entry.OriginalValue is JsonElement je && je.ValueKind == JsonValueKind.Number)
                            {
                                subKey.SetValue(entry.ValueName, je.GetInt32(), kind);
                            }
                            else if (entry.OriginalValue != null)
                            {
                                subKey.SetValue(entry.ValueName, entry.OriginalValue, kind);
                            }
                            Log($"[Rollback] Restored original value: {hive}\\{entry.SubKey}\\{entry.ValueName} = {entry.OriginalValue}");
                        }
                    }
                }
                catch (Exception ex)
                {
                    Log($"[Rollback Error] Could not revert {entry.RegistryHive}\\{entry.SubKey}\\{entry.ValueName}: {ex.Message}");
                    success = false;
                }
            }

            _trackedChanges.Clear();
            SaveTrackedChanges();
            return success;
        });
    }

    private void TrackCurrentValue(RegistryHive hive, string subKeyPath, string valueName)
    {
        try
        {
            if (_trackedChanges.Any(x => x.RegistryHive == hive.ToString() &&
                                         string.Equals(x.SubKey, subKeyPath, StringComparison.OrdinalIgnoreCase) &&
                                         string.Equals(x.ValueName, valueName, StringComparison.OrdinalIgnoreCase)))
            {
                return;
            }

            using var baseKey = RegistryKey.OpenBaseKey(hive, RegistryView.Registry64);
            using var subKey = baseKey.OpenSubKey(subKeyPath, false);

            var entry = new RegistryValueBackup
            {
                RegistryHive = hive.ToString(),
                SubKey = subKeyPath,
                ValueName = valueName,
                ExistedBefore = false
            };

            if (subKey != null)
            {
                var val = subKey.GetValue(valueName);
                if (val != null)
                {
                    entry.ExistedBefore = true;
                    entry.OriginalValue = val;
                    entry.ValueKind = subKey.GetValueKind(valueName).ToString();
                }
            }

            _trackedChanges.Add(entry);
            SaveTrackedChanges();
        }
        catch (Exception ex)
        {
            Log($"[Registry Backup Warning] Failed to snapshot {hive}\\{subKeyPath}\\{valueName}: {ex.Message}");
        }
    }

    private void SaveTrackedChanges()
    {
        try
        {
            string json = JsonSerializer.Serialize(_trackedChanges, new JsonSerializerOptions { WriteIndented = true });
            File.WriteAllText(_backupStorePath, json);
        }
        catch { }
    }

    private void LoadTrackedChanges()
    {
        try
        {
            if (File.Exists(_backupStorePath))
            {
                string json = File.ReadAllText(_backupStorePath);
                var items = JsonSerializer.Deserialize<List<RegistryValueBackup>>(json);
                if (items != null)
                {
                    _trackedChanges.Clear();
                    _trackedChanges.AddRange(items);
                }
            }
        }
        catch { }
    }
}
