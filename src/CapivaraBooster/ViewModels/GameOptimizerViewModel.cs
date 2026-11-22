using System.Windows;
using System.Windows.Input;
using CapivaraBooster.Common;
using CapivaraBooster.Models;
using CapivaraBooster.Services;

namespace CapivaraBooster.ViewModels;

public class GameOptimizerViewModel : ObservableObject
{
    private readonly IGameOptimizerService _gameService;
    private readonly IRestoreService _restoreService;

    private int _selectedGameIndex = 0;
    private bool _isBusy;

    public LocalizationService Loc => LocalizationService.Instance;

    // CS2
    private GameOptimizationState _cs2State = new() { GameId = "cs2", DisplayName = "Counter-Strike 2", ExeName = "cs2.exe", IconColorHex = "#FFB300" };
    private bool _cs2ClearShaderCache = true;
    private bool _cs2GenerateAutoexec = true;
    private string _cs2LaunchOptions = string.Empty;

    // Valorant
    private GameOptimizationState _valState = new() { GameId = "valorant", DisplayName = "Valorant", ExeName = "VALORANT-Win64-Shipping.exe", IconColorHex = "#FF1744" };
    private bool _valLowPriorityClient = true;
    private bool _valClearLogs = true;
    private bool _valDefenderExclusions = true;

    // League of Legends
    private GameOptimizationState _lolState = new() { GameId = "lol", DisplayName = "League of Legends", ExeName = "League of Legends.exe", IconColorHex = "#00E5FF" };
    private bool _lolLowSpecAndCloseClient = true;
    private bool _lolClearLogs = true;
    private bool _lolDefenderExclusions = true;

    public GameOptimizerViewModel(IGameOptimizerService gameService, IRestoreService restoreService)
    {
        _gameService = gameService;
        _restoreService = restoreService;

        _cs2LaunchOptions = _gameService.GetCs2RecommendedLaunchOptions();

        DetectAllGamesCommand = new RelayCommand(async () => await DetectAllGamesAsync(), () => !IsBusy);
        OptimizeCs2Command = new RelayCommand(async () => await OptimizeCs2Async(), () => !IsBusy);
        OptimizeValorantCommand = new RelayCommand(async () => await OptimizeValorantAsync(), () => !IsBusy);
        OptimizeLolCommand = new RelayCommand(async () => await OptimizeLolAsync(), () => !IsBusy);

        RevertCs2Command = new RelayCommand(async () => await RevertGameAsync("cs2", Cs2State.InstalledPath), () => !IsBusy);
        RevertValorantCommand = new RelayCommand(async () => await RevertGameAsync("valorant", ValState.InstalledPath), () => !IsBusy);
        RevertLolCommand = new RelayCommand(async () => await RevertGameAsync("lol", LolState.InstalledPath), () => !IsBusy);

        CopyCs2LaunchOptionsCommand = new RelayCommand(CopyCs2LaunchOptions);
    }

    public int SelectedGameIndex
    {
        get => _selectedGameIndex;
        set => SetProperty(ref _selectedGameIndex, value);
    }

    public bool IsBusy
    {
        get => _isBusy;
        set
        {
            if (SetProperty(ref _isBusy, value))
            {
                (DetectAllGamesCommand as RelayCommand)?.RaiseCanExecuteChanged();
                (OptimizeCs2Command as RelayCommand)?.RaiseCanExecuteChanged();
                (OptimizeValorantCommand as RelayCommand)?.RaiseCanExecuteChanged();
                (OptimizeLolCommand as RelayCommand)?.RaiseCanExecuteChanged();
            }
        }
    }

    public GameOptimizationState Cs2State
    {
        get => _cs2State;
        set => SetProperty(ref _cs2State, value);
    }

    public bool Cs2ClearShaderCache
    {
        get => _cs2ClearShaderCache;
        set => SetProperty(ref _cs2ClearShaderCache, value);
    }

    public bool Cs2GenerateAutoexec
    {
        get => _cs2GenerateAutoexec;
        set => SetProperty(ref _cs2GenerateAutoexec, value);
    }

    public string Cs2LaunchOptions
    {
        get => _cs2LaunchOptions;
        set => SetProperty(ref _cs2LaunchOptions, value);
    }

    public GameOptimizationState ValState
    {
        get => _valState;
        set => SetProperty(ref _valState, value);
    }

    public bool ValLowPriorityClient
    {
        get => _valLowPriorityClient;
        set => SetProperty(ref _valLowPriorityClient, value);
    }

    public bool ValClearLogs
    {
        get => _valClearLogs;
        set => SetProperty(ref _valClearLogs, value);
    }

    public bool ValDefenderExclusions
    {
        get => _valDefenderExclusions;
        set => SetProperty(ref _valDefenderExclusions, value);
    }

    public GameOptimizationState LolState
    {
        get => _lolState;
        set => SetProperty(ref _lolState, value);
    }

    public bool LolLowSpecAndCloseClient
    {
        get => _lolLowSpecAndCloseClient;
        set => SetProperty(ref _lolLowSpecAndCloseClient, value);
    }

    public bool LolClearLogs
    {
        get => _lolClearLogs;
        set => SetProperty(ref _lolClearLogs, value);
    }

    public bool LolDefenderExclusions
    {
        get => _lolDefenderExclusions;
        set => SetProperty(ref _lolDefenderExclusions, value);
    }

    public ICommand DetectAllGamesCommand { get; }
    public ICommand OptimizeCs2Command { get; }
    public ICommand OptimizeValorantCommand { get; }
    public ICommand OptimizeLolCommand { get; }
    public ICommand RevertCs2Command { get; }
    public ICommand RevertValorantCommand { get; }
    public ICommand RevertLolCommand { get; }
    public ICommand CopyCs2LaunchOptionsCommand { get; }

    public async Task DetectAllGamesAsync()
    {
        IsBusy = true;
        try
        {
            var cs2 = await _gameService.DetectGameAsync("cs2");
            Cs2State.IsDetected = cs2.Found;
            Cs2State.InstalledPath = cs2.ExecutablePath;

            var val = await _gameService.DetectGameAsync("valorant");
            ValState.IsDetected = val.Found;
            ValState.InstalledPath = val.ExecutablePath;

            var lol = await _gameService.DetectGameAsync("lol");
            LolState.IsDetected = lol.Found;
            LolState.InstalledPath = lol.ExecutablePath;
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task OptimizeCs2Async()
    {
        IsBusy = true;
        try
        {
            await _restoreService.CreateSystemRestorePointAsync("CapivaraBooster_PreCS2Opt");
            bool res = await _gameService.OptimizeCs2Async(Cs2State.InstalledPath, Cs2GenerateAutoexec, Cs2ClearShaderCache);
            Cs2State.HighPriorityApplied = true;
            Cs2State.DisableCore0Applied = true;
            Cs2State.DisableFseApplied = true;
            Cs2State.HighDpiApplied = true;
            Cs2State.GameSpecificApplied = res;
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task OptimizeValorantAsync()
    {
        IsBusy = true;
        try
        {
            await _restoreService.CreateSystemRestorePointAsync("CapivaraBooster_PreValorantOpt");
            bool res = await _gameService.OptimizeValorantAsync(ValState.InstalledPath, ValLowPriorityClient, ValClearLogs, ValDefenderExclusions);
            ValState.HighPriorityApplied = true;
            ValState.DisableCore0Applied = true;
            ValState.DisableFseApplied = true;
            ValState.HighDpiApplied = true;
            ValState.GameSpecificApplied = res;
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task OptimizeLolAsync()
    {
        IsBusy = true;
        try
        {
            await _restoreService.CreateSystemRestorePointAsync("CapivaraBooster_PreLoLOpt");
            bool res = await _gameService.OptimizeLeagueOfLegendsAsync(LolState.InstalledPath, LolLowSpecAndCloseClient, LolClearLogs, LolDefenderExclusions);
            LolState.HighPriorityApplied = true;
            LolState.DisableFseApplied = true;
            LolState.HighDpiApplied = true;
            LolState.GameSpecificApplied = res;
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task RevertGameAsync(string gameId, string exePath)
    {
        IsBusy = true;
        try
        {
            await _gameService.RevertGameTweaksAsync(gameId, exePath);
            if (gameId == "cs2")
            {
                Cs2State.HighPriorityApplied = false;
                Cs2State.DisableCore0Applied = false;
                Cs2State.DisableFseApplied = false;
                Cs2State.HighDpiApplied = false;
                Cs2State.GameSpecificApplied = false;
            }
            else if (gameId == "valorant")
            {
                ValState.HighPriorityApplied = false;
                ValState.DisableCore0Applied = false;
                ValState.DisableFseApplied = false;
                ValState.HighDpiApplied = false;
                ValState.GameSpecificApplied = false;
            }
            else if (gameId == "lol")
            {
                LolState.HighPriorityApplied = false;
                LolState.DisableFseApplied = false;
                LolState.HighDpiApplied = false;
                LolState.GameSpecificApplied = false;
            }
        }
        finally
        {
            IsBusy = false;
        }
    }

    private void CopyCs2LaunchOptions()
    {
        try
        {
            Clipboard.SetText(Cs2LaunchOptions);
        }
        catch { }
    }
}
