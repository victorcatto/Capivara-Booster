using System.ComponentModel;

namespace CapivaraBooster.Services;

public interface ILocalizationService : INotifyPropertyChanged
{
    string CurrentLanguage { get; }
    bool IsPortuguese { get; }
    bool IsEnglish { get; }

    string this[string key] { get; }
    string GetString(string key);
    void SetLanguage(string languageCode);
}
