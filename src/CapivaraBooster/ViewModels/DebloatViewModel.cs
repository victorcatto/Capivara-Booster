using System.Collections.ObjectModel;
using System.Windows.Input;
using CapivaraBooster.Common;
using CapivaraBooster.Models;
using CapivaraBooster.Services;

namespace CapivaraBooster.ViewModels;

public class DebloatViewModel : ObservableObject
{
    private readonly IDebloatService _debloatService;
    private readonly IRestoreService _restoreService;
    private bool _isBusy;
    private bool _isTelemetryDisabled;
    private string _filterText = string.Empty;

    public LocalizationService Loc => LocalizationService.Instance;

    public DebloatViewModel(IDebloatService debloatService, IRestoreService restoreService)
    {
        _debloatService = debloatService;
        _restoreService = restoreService;

        BloatwareItems = new ObservableCollection<BloatwareItem>(_debloatService.GetDefaultBloatwareList());

        SelectAllSafeCommand = new RelayCommand(SelectAllSafe);
        DeselectAllCommand = new RelayCommand(DeselectAll);
        RemoveSelectedCommand = new RelayCommand(async () => await RemoveSelectedAsync(), () => !IsBusy);
        ToggleTelemetryCommand = new RelayCommand(async () => await ToggleTelemetryAsync(), () => !IsBusy);
    }

    public ObservableCollection<BloatwareItem> BloatwareItems { get; }

    public bool IsBusy
    {
        get => _isBusy;
        set
        {
            if (SetProperty(ref _isBusy, value))
            {
                (RemoveSelectedCommand as RelayCommand)?.RaiseCanExecuteChanged();
                (ToggleTelemetryCommand as RelayCommand)?.RaiseCanExecuteChanged();
            }
        }
    }

    public bool IsTelemetryDisabled
    {
        get => _isTelemetryDisabled;
        set => SetProperty(ref _isTelemetryDisabled, value);
    }

    public string FilterText
    {
        get => _filterText;
        set => SetProperty(ref _filterText, value);
    }

    public ICommand SelectAllSafeCommand { get; }
    public ICommand DeselectAllCommand { get; }
    public ICommand RemoveSelectedCommand { get; }
    public ICommand ToggleTelemetryCommand { get; }

    private void SelectAllSafe()
    {
        foreach (var item in BloatwareItems)
        {
            item.IsSelected = item.IsSafe;
        }
    }

    private void DeselectAll()
    {
        foreach (var item in BloatwareItems)
        {
            item.IsSelected = false;
        }
    }

    private async Task RemoveSelectedAsync()
    {
        IsBusy = true;
        try
        {
            await _restoreService.CreateSystemRestorePointAsync("CapivaraBooster_PreDebloat");
            await _debloatService.RemoveSelectedBloatwareAsync(BloatwareItems);
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task ToggleTelemetryAsync()
    {
        IsBusy = true;
        try
        {
            if (!IsTelemetryDisabled)
            {
                await _debloatService.DisableTelemetryAndTrackingAsync();
                IsTelemetryDisabled = true;
            }
            else
            {
                await _debloatService.EnableTelemetryAndTrackingAsync();
                IsTelemetryDisabled = false;
            }
        }
        finally
        {
            IsBusy = false;
        }
    }
}
