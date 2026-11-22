using System.Windows.Input;
using CapivaraBooster.Common;
using CapivaraBooster.Services;

namespace CapivaraBooster.ViewModels;

public class DashboardViewModel : ObservableObject
{
    private readonly ISystemBoostService _boostService;
    private readonly ICleanerService _cleanerService;
    private readonly IRestoreService _restoreService;
    private readonly IGameOptimizerService _gameService;

    private string _osVersion = string.Empty;
    private string _hpetStatus = "Checking...";
    private string _powerScheme = "Checking...";
    private string _gamesDetectedSummary = "Checking...";
    private bool _isBoosting;
    private double _boostProgress;

    public LocalizationService Loc => LocalizationService.Instance;

    public DashboardViewModel(
        ISystemBoostService boostService,
        ICleanerService cleanerService,
        IRestoreService restoreService,
        IGameOptimizerService gameService)
    {
        _boostService = boostService;
        _cleanerService = cleanerService;
        _restoreService = restoreService;
        _gameService = gameService;

        QuickBoostCommand = new RelayCommand(async () => await ExecuteQuickBoostAsync(), () => !IsBoosting);
        RefreshDashboardCommand = new RelayCommand(async () => await RefreshStatusAsync());

        _osVersion = $"{Environment.OSVersion.VersionString} ({(Environment.Is64BitOperatingSystem ? "64-bit" : "32-bit")})";
    }

    public string OsVersion
    {
        get => _osVersion;
        set => SetProperty(ref _osVersion, value);
    }

    public string HpetStatus
    {
        get => _hpetStatus;
        set => SetProperty(ref _hpetStatus, value);
    }

    public string PowerScheme
    {
        get => _powerScheme;
        set => SetProperty(ref _powerScheme, value);
    }

    public string GamesDetectedSummary
    {
        get => _gamesDetectedSummary;
        set => SetProperty(ref _gamesDetectedSummary, value);
    }

    public bool IsBoosting
    {
        get => _isBoosting;
        set
        {
            if (SetProperty(ref _isBoosting, value))
            {
                (QuickBoostCommand as RelayCommand)?.RaiseCanExecuteChanged();
            }
        }
    }

    public double BoostProgress
    {
        get => _boostProgress;
        set => SetProperty(ref _boostProgress, value);
    }

    public ICommand QuickBoostCommand { get; }
    public ICommand RefreshDashboardCommand { get; }

    public async Task RefreshStatusAsync()
    {
        bool hpetOff = await _boostService.IsHpetDisabledAsync();
        HpetStatus = hpetOff ? "Optimized (Disabled)" : "Default (Enabled)";

        var cs2 = await _gameService.DetectGameAsync("cs2");
        var val = await _gameService.DetectGameAsync("valorant");
        var lol = await _gameService.DetectGameAsync("lol");

        int count = (cs2.Found ? 1 : 0) + (val.Found ? 1 : 0) + (lol.Found ? 1 : 0);
        GamesDetectedSummary = $"{count} of 3 (CS2: {(cs2.Found ? "✔" : "✘")}, Valorant: {(val.Found ? "✔" : "✘")}, LoL: {(lol.Found ? "✔" : "✘")})";
        PowerScheme = "Configured";
    }

    private async Task ExecuteQuickBoostAsync()
    {
        IsBoosting = true;
        BoostProgress = 10;

        try
        {
            await _restoreService.CreateSystemRestorePointAsync("CapivaraBooster_QuickBoost");
            BoostProgress = 30;

            await _boostService.EnableUltimatePerformancePowerPlanAsync();
            BoostProgress = 50;

            await _boostService.ApplySystemOptimizationsAsync();
            BoostProgress = 70;

            var items = _cleanerService.GetCleanerItems();
            await _cleanerService.CleanSelectedAsync(items);
            BoostProgress = 90;

            await RefreshStatusAsync();
            BoostProgress = 100;
        }
        finally
        {
            IsBoosting = false;
        }
    }
}
