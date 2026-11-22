using System.ComponentModel;
using System.Globalization;
using System.IO;
using System.Runtime.CompilerServices;
using System.Text.Json;

namespace CapivaraBooster.Services;

public class LocalizationService : ILocalizationService
{
    public static LocalizationService Instance { get; } = new LocalizationService();

    public event PropertyChangedEventHandler? PropertyChanged;

    private readonly Dictionary<string, string> _strings = new(StringComparer.OrdinalIgnoreCase);
    private string _currentLanguage = "pt-BR";
    private readonly string _settingsFilePath;

    public string CurrentLanguage => _currentLanguage;
    public bool IsPortuguese => string.Equals(_currentLanguage, "pt-BR", StringComparison.OrdinalIgnoreCase);
    public bool IsEnglish => string.Equals(_currentLanguage, "en-US", StringComparison.OrdinalIgnoreCase);

    public string this[string key] => GetString(key);

    private LocalizationService()
    {
        string appData = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "CapivaraBooster");
        Directory.CreateDirectory(appData);
        _settingsFilePath = Path.Combine(appData, "settings.json");

        string initialLang = DetectInitialLanguage();
        SetLanguage(initialLang);
    }

    public string GetString(string key)
    {
        if (_strings.TryGetValue(key, out var val))
        {
            return val;
        }

        return $"[{key}]";
    }

    public void SetLanguage(string languageCode)
    {
        if (string.IsNullOrWhiteSpace(languageCode)) return;

        string target = languageCode.ToLowerInvariant().StartsWith("pt") ? "pt-BR" : "en-US";
        _currentLanguage = target;

        LoadDictionary(target);
        SaveSettings();

        // Notify WPF binding system that all indexer values and properties changed
        OnPropertyChanged(string.Empty);
        OnPropertyChanged("Item[]");
        OnPropertyChanged(nameof(CurrentLanguage));
        OnPropertyChanged(nameof(IsPortuguese));
        OnPropertyChanged(nameof(IsEnglish));
    }

    private void LoadDictionary(string langCode)
    {
        string baseDir = AppDomain.CurrentDomain.BaseDirectory;
        string candidate1 = Path.Combine(baseDir, "Localization", $"{langCode}.json");
        string candidate2 = Path.Combine(baseDir, "..", "..", "..", "Localization", $"{langCode}.json");
        string targetFile = File.Exists(candidate1) ? candidate1 : (File.Exists(candidate2) ? candidate2 : candidate1);

        if (File.Exists(targetFile))
        {
            try
            {
                string json = File.ReadAllText(targetFile);
                var dict = JsonSerializer.Deserialize<Dictionary<string, string>>(json);
                if (dict != null)
                {
                    _strings.Clear();
                    foreach (var kvp in dict)
                    {
                        _strings[kvp.Key] = kvp.Value;
                    }
                }
            }
            catch { /* retain existing on failure */ }
        }
    }

    private string DetectInitialLanguage()
    {
        // 1. Check saved settings
        if (File.Exists(_settingsFilePath))
        {
            try
            {
                string json = File.ReadAllText(_settingsFilePath);
                using var doc = JsonDocument.Parse(json);
                if (doc.RootElement.TryGetProperty("Language", out var langProp))
                {
                    string? saved = langProp.GetString();
                    if (!string.IsNullOrEmpty(saved)) return saved;
                }
            }
            catch { }
        }

        // 2. Auto-detect Windows system language
        string cultureName = CultureInfo.CurrentUICulture.Name;
        if (cultureName.StartsWith("pt", StringComparison.OrdinalIgnoreCase))
        {
            return "pt-BR";
        }

        return "en-US";
    }

    private void SaveSettings()
    {
        try
        {
            var data = new Dictionary<string, string>
            {
                ["Language"] = _currentLanguage
            };
            string json = JsonSerializer.Serialize(data, new JsonSerializerOptions { WriteIndented = true });
            File.WriteAllText(_settingsFilePath, json);
        }
        catch { }
    }

    private void OnPropertyChanged([CallerMemberName] string? propertyName = null)
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
}
