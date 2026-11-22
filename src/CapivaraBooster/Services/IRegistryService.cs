using CapivaraBooster.Models;
using Microsoft.Win32;

namespace CapivaraBooster.Services;

public interface IRegistryService
{
    event Action<string>? LogMessage;

    bool SetDword(RegistryHive hive, string subKeyPath, string valueName, int value, bool trackForRestore = true);
    bool SetString(RegistryHive hive, string subKeyPath, string valueName, string value, bool trackForRestore = true);
    bool DeleteValue(RegistryHive hive, string subKeyPath, string valueName, bool trackForRestore = true);
    bool DeleteKeyTree(RegistryHive hive, string subKeyPath);
    object? GetValue(RegistryHive hive, string subKeyPath, string valueName);
    int? GetDword(RegistryHive hive, string subKeyPath, string valueName);
    string? GetString(RegistryHive hive, string subKeyPath, string valueName);
    bool KeyExists(RegistryHive hive, string subKeyPath);
    IReadOnlyList<RegistryValueBackup> GetTrackedBackups();
    Task<bool> RevertAllTrackedAsync();
}
