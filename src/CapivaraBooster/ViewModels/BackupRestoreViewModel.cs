using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Windows.Input;
using CapivaraBooster.Common;
using CapivaraBooster.Models;
using CapivaraBooster.Services;

namespace CapivaraBooster.ViewModels;

public class BackupRestoreViewModel : ObservableObject
{
    private readonly IRestoreService _restoreService;
    private readonly IRegistryService _registryService;
    private bool _isBusy;
    private BackupRecord? _selectedBackup;
    private int _trackedRegistryKeysCount;

    public LocalizationService Loc => LocalizationService.Instance;

    public BackupRestoreViewModel(IRestoreService restoreService, IRegistryService registryService)
    {
        _restoreService = restoreService;
        _registryService = registryService;

        BackupRecords = new ObservableCollection<BackupRecord>(_restoreService.GetBackups());
        _trackedRegistryKeysCount = _registryService.GetTrackedBackups().Count;

        CreateRestorePointCommand = new RelayCommand(async () => await CreateRestorePointAsync(), () => !IsBusy);
        RestoreBackupFileCommand = new RelayCommand(async () => await RestoreSelectedFileAsync(), () => !IsBusy && SelectedBackup != null);
        RevertRegistryCommand = new RelayCommand(async () => await RevertRegistryAsync(), () => !IsBusy && TrackedRegistryKeysCount > 0);
        OpenBackupFolderCommand = new RelayCommand(OpenBackupFolder);
    }

    public ObservableCollection<BackupRecord> BackupRecords { get; }

    public BackupRecord? SelectedBackup
    {
        get => _selectedBackup;
        set
        {
            if (SetProperty(ref _selectedBackup, value))
            {
                (RestoreBackupFileCommand as RelayCommand)?.RaiseCanExecuteChanged();
            }
        }
    }

    public int TrackedRegistryKeysCount
    {
        get => _trackedRegistryKeysCount;
        set => SetProperty(ref _trackedRegistryKeysCount, value);
    }

    public bool IsBusy
    {
        get => _isBusy;
        set
        {
            if (SetProperty(ref _isBusy, value))
            {
                (CreateRestorePointCommand as RelayCommand)?.RaiseCanExecuteChanged();
                (RestoreBackupFileCommand as RelayCommand)?.RaiseCanExecuteChanged();
                (RevertRegistryCommand as RelayCommand)?.RaiseCanExecuteChanged();
            }
        }
    }

    public ICommand CreateRestorePointCommand { get; }
    public ICommand RestoreBackupFileCommand { get; }
    public ICommand RevertRegistryCommand { get; }
    public ICommand OpenBackupFolderCommand { get; }

    public void Refresh()
    {
        BackupRecords.Clear();
        foreach (var b in _restoreService.GetBackups())
        {
            BackupRecords.Add(b);
        }
        TrackedRegistryKeysCount = _registryService.GetTrackedBackups().Count;
    }

    private async Task CreateRestorePointAsync()
    {
        IsBusy = true;
        try
        {
            string desc = $"CapivaraBooster_Manual_{DateTime.Now:yyyyMMdd_HHmm}";
            await _restoreService.CreateSystemRestorePointAsync(desc);
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task RestoreSelectedFileAsync()
    {
        if (SelectedBackup == null) return;

        IsBusy = true;
        try
        {
            await _restoreService.RestoreFileAsync(SelectedBackup.Id);
            Refresh();
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task RevertRegistryAsync()
    {
        IsBusy = true;
        try
        {
            await _registryService.RevertAllTrackedAsync();
            TrackedRegistryKeysCount = _registryService.GetTrackedBackups().Count;
        }
        finally
        {
            IsBusy = false;
        }
    }

    private void OpenBackupFolder()
    {
        try
        {
            string folder = _restoreService.GetBackupFolder();
            Process.Start(new ProcessStartInfo
            {
                FileName = folder,
                UseShellExecute = true
            });
        }
        catch { }
    }
}
