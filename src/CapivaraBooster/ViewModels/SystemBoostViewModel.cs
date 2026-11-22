using System.Collections.ObjectModel;
using System.Windows.Input;
using CapivaraBooster.Common;
using CapivaraBooster.Services;

namespace CapivaraBooster.ViewModels;

public class SystemBoostViewModel : ObservableObject
{
    private readonly ISystemBoostService _boostService;
    private readonly IRestoreService _restoreService;

    private bool _isBusy;
    private bool _isHpetDisabled;
    private bool _isUltimatePlanActive;
    private bool _areTweaksApplied;
    private string _activePowerPlan = "Checking...";

    public LocalizationService Loc => LocalizationService.Instance;

    public SystemBoostViewModel(ISystemBoostService boostService, IRestoreService restoreService)
    {
        _boostService = boostService;
        _restoreService = restoreService;

        StartupItems = new ObservableCollection<StartupItem>();

        ToggleHpetCommand = new RelayCommand(async () => await ToggleHpetAsync(), () => !IsBusy);
        EnableUltimatePowerCommand = new RelayCommand(async () => await EnableUltimatePowerAsync(), () => !IsBusy);
        ApplyAllOptimizationsCommand = new RelayCommand(async () => await ApplyAllOptimizationsAsync(), () => !IsBusy);
        RevertOptimizationsCommand = new RelayCommand(async () => await RevertOptimizationsAsync(), () => !IsBusy);
        RefreshStartupItemsCommand = new RelayCommand(async () => await RefreshStartupAsync(), () => !IsBusy);
    }

    public ObservableCollection<StartupItem> StartupItems { get; }

    public bool IsBusy
    {
        get => _isBusy;
        set
        {
            if (SetProperty(ref _isBusy, value))
            {
                (ToggleHpetCommand as RelayCommand)?.RaiseCanExecuteChanged();
                (EnableUltimatePowerCommand as RelayCommand)?.RaiseCanExecuteChanged();
                (ApplyAllOptimizationsCommand as RelayCommand)?.RaiseCanExecuteChanged();
                (RevertOptimizationsCommand as RelayCommand)?.RaiseCanExecuteChanged();
            }
        }
    }

    public bool IsHpetDisabled
    {
        get => _isHpetDisabled;
        set => SetProperty(ref _isHpetDisabled, value);
    }

    public bool IsUltimatePlanActive
    {
        get => _isUltimatePlanActive;
        set => SetProperty(ref _isUltimatePlanActive, value);
    }

    public bool AreTweaksApplied
    {
        get => _areTweaksApplied;
        set => SetProperty(ref _areTweaksApplied, value);
    }

    public string ActivePowerPlan
    {
        get => _activePowerPlan;
        set => SetProperty(ref _activePowerPlan, value);
    }

    public ICommand ToggleHpetCommand { get; }
    public ICommand EnableUltimatePowerCommand { get; }
    public ICommand ApplyAllOptimizationsCommand { get; }
    public ICommand RevertOptimizationsCommand { get; }
    public ICommand RefreshStartupItemsCommand { get; }

    public async Task InitializeAsync()
    {
        IsHpetDisabled = await _boostService.IsHpetDisabledAsync();
        await RefreshStartupAsync();
    }

    private async Task ToggleHpetAsync()
    {
        IsBusy = true;
        try
        {
            bool targetState = !IsHpetDisabled;
            await _boostService.SetHpetStateAsync(targetState);
            IsHpetDisabled = await _boostService.IsHpetDisabledAsync();
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task EnableUltimatePowerAsync()
    {
        IsBusy = true;
        try
        {
            string guid = await _boostService.EnableUltimatePerformancePowerPlanAsync();
            ActivePowerPlan = $"Ultimate Performance ({guid.Substring(0, 8)}...)";
            IsUltimatePlanActive = true;
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task ApplyAllOptimizationsAsync()
    {
        IsBusy = true;
        try
        {
            await _restoreService.CreateSystemRestorePointAsync("CapivaraBooster_PreSystemBoost");
            await _boostService.ApplySystemOptimizationsAsync();
            AreTweaksApplied = true;
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task RevertOptimizationsAsync()
    {
        IsBusy = true;
        try
        {
            await _boostService.RevertSystemOptimizationsAsync();
            AreTweaksApplied = false;
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task RefreshStartupAsync()
    {
        var items = await _boostService.GetStartupItemsAsync();
        StartupItems.Clear();
        foreach (var item in items)
        {
            StartupItems.Add(item);
        }
    }
}
