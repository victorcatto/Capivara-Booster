using System.Collections.ObjectModel;
using System.Windows.Input;
using CapivaraBooster.Common;
using CapivaraBooster.Models;
using CapivaraBooster.Services;

namespace CapivaraBooster.ViewModels;

public class CleanerViewModel : ObservableObject
{
    private readonly ICleanerService _cleanerService;
    private bool _isBusy;
    private long _totalFoundBytes;

    public LocalizationService Loc => LocalizationService.Instance;

    public CleanerViewModel(ICleanerService cleanerService)
    {
        _cleanerService = cleanerService;
        CleanerItems = new ObservableCollection<CleanerItem>(_cleanerService.GetCleanerItems());

        ScanCommand = new RelayCommand(async () => await ScanAsync(), () => !IsBusy);
        CleanCommand = new RelayCommand(async () => await CleanAsync(), () => !IsBusy);
        FlushDnsCommand = new RelayCommand(async () => await FlushDnsAsync(), () => !IsBusy);
        SelectAllCommand = new RelayCommand(() => SetSelection(true));
        DeselectAllCommand = new RelayCommand(() => SetSelection(false));
    }

    public ObservableCollection<CleanerItem> CleanerItems { get; }

    public bool IsBusy
    {
        get => _isBusy;
        set
        {
            if (SetProperty(ref _isBusy, value))
            {
                (ScanCommand as RelayCommand)?.RaiseCanExecuteChanged();
                (CleanCommand as RelayCommand)?.RaiseCanExecuteChanged();
                (FlushDnsCommand as RelayCommand)?.RaiseCanExecuteChanged();
            }
        }
    }

    public long TotalFoundBytes
    {
        get => _totalFoundBytes;
        set
        {
            if (SetProperty(ref _totalFoundBytes, value))
            {
                OnPropertyChanged(nameof(FormattedTotalFound));
            }
        }
    }

    public string FormattedTotalFound
    {
        get
        {
            if (TotalFoundBytes < 1024) return $"{TotalFoundBytes} B";
            if (TotalFoundBytes < 1024 * 1024) return $"{TotalFoundBytes / 1024.0:F1} KB";
            if (TotalFoundBytes < 1024 * 1024 * 1024) return $"{TotalFoundBytes / (1024.0 * 1024.0):F1} MB";
            return $"{TotalFoundBytes / (1024.0 * 1024.0 * 1024.0):F2} GB";
        }
    }

    public ICommand ScanCommand { get; }
    public ICommand CleanCommand { get; }
    public ICommand FlushDnsCommand { get; }
    public ICommand SelectAllCommand { get; }
    public ICommand DeselectAllCommand { get; }

    private void SetSelection(bool select)
    {
        foreach (var item in CleanerItems)
        {
            item.IsSelected = select;
        }
    }

    public async Task ScanAsync()
    {
        IsBusy = true;
        try
        {
            await _cleanerService.ScanSizesAsync(CleanerItems);
            TotalFoundBytes = CleanerItems.Sum(x => x.SizeBytes);
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task CleanAsync()
    {
        IsBusy = true;
        try
        {
            var summary = await _cleanerService.CleanSelectedAsync(CleanerItems);
            await ScanAsync();
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task FlushDnsAsync()
    {
        IsBusy = true;
        try
        {
            await _cleanerService.FlushDnsCacheAsync();
        }
        finally
        {
            IsBusy = false;
        }
    }
}
