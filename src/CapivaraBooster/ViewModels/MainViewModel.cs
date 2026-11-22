using System.Collections.Concurrent;
using System.Collections.ObjectModel;
using System.Text;
using System.Windows;
using System.Windows.Input;
using System.Windows.Threading;
using CapivaraBooster.Common;
using CapivaraBooster.Models;
using CapivaraBooster.Services;

namespace CapivaraBooster.ViewModels;

public enum NavigationTab
{
    Dashboard,
    Debloat,
    Cleaner,
    SystemBoost,
    GameOptimizer,
    BackupRestore
}

public class MainViewModel : ObservableObject
{
    private NavigationTab _currentTab = NavigationTab.Dashboard;
    private string _statusText = "Ready";
    private bool _isGlobalBusy;

    // Auth Modal & State
    private bool _isAuthModalOpen;
    private bool _isRegisterMode;
    private string _authEmail = string.Empty;
    private string _authPassword = string.Empty;
    private string _authErrorMessage = string.Empty;

    // Update State
    private bool _isUpdateBannerVisible;
    private string _updateMessage = string.Empty;

    // High-performance log batching queue and timer
    private readonly ConcurrentQueue<LogItem> _logQueue = new();
    private readonly DispatcherTimer _logFlushTimer;

    public LocalizationService Loc => LocalizationService.Instance;
    public IAuthService AuthService { get; }
    public IUpdateService UpdateService { get; }

    public DashboardViewModel DashboardVm { get; }
    public DebloatViewModel DebloatVm { get; }
    public CleanerViewModel CleanerVm { get; }
    public SystemBoostViewModel SystemBoostVm { get; }
    public GameOptimizerViewModel GameOptimizerVm { get; }
    public BackupRestoreViewModel BackupRestoreVm { get; }

    public ObservableCollection<LogItem> LogItems { get; }

    public MainViewModel(
        DashboardViewModel dashboardVm,
        DebloatViewModel debloatVm,
        CleanerViewModel cleanerVm,
        SystemBoostViewModel systemBoostVm,
        GameOptimizerViewModel gameOptimizerVm,
        BackupRestoreViewModel backupRestoreVm,
        IProcessExecutor executor,
        IRegistryService registryService,
        IRestoreService restoreService,
        IDebloatService debloatService,
        ICleanerService cleanerService,
        ISystemBoostService boostService,
        IGameOptimizerService gameService,
        IAuthService authService,
        IUpdateService updateService)
    {
        DashboardVm = dashboardVm;
        DebloatVm = debloatVm;
        CleanerVm = cleanerVm;
        SystemBoostVm = systemBoostVm;
        GameOptimizerVm = gameOptimizerVm;
        BackupRestoreVm = backupRestoreVm;
        AuthService = authService;
        UpdateService = updateService;

        LogItems = new ObservableCollection<LogItem>();

        // Flush log queue every 40ms to eliminate UI thread dispatch overhead
        _logFlushTimer = new DispatcherTimer(DispatcherPriority.Background)
        {
            Interval = TimeSpan.FromMilliseconds(40)
        };
        _logFlushTimer.Tick += FlushLogQueue;
        _logFlushTimer.Start();

        // Navigation Command
        NavigateCommand = new RelayCommand(p =>
        {
            if (p is NavigationTab tab) CurrentTab = tab;
            else if (p is string s && Enum.TryParse<NavigationTab>(s, out var parsed)) CurrentTab = parsed;
        });

        ClearLogsCommand = new RelayCommand(ClearLogs);
        CopyLogsCommand = new RelayCommand(CopyLogs);

        // Language Switcher Command
        ChangeLanguageCommand = new RelayCommand(p =>
        {
            string lang = p?.ToString() ?? "pt-BR";
            Loc.SetLanguage(lang);
            AddLog(Loc.IsPortuguese
                ? $"Idioma alterado para Português (Brasil)."
                : $"Language changed to English (US).", LogLevel.Info);
        });

        // Auth Commands
        OpenAuthModalCommand = new RelayCommand(() =>
        {
            AuthErrorMessage = string.Empty;
            IsAuthModalOpen = true;
        });

        CloseAuthModalCommand = new RelayCommand(() => IsAuthModalOpen = false);

        SwitchAuthModeCommand = new RelayCommand(() =>
        {
            IsRegisterMode = !IsRegisterMode;
            AuthErrorMessage = string.Empty;
        });

        LoginCommand = new RelayCommand(async () => await ExecuteLoginAsync());
        RegisterCommand = new RelayCommand(async () => await ExecuteRegisterAsync());
        LogoutCommand = new RelayCommand(() =>
        {
            AuthService.Logout();
            AddLog(Loc.IsPortuguese ? "Sessão encerrada." : "Logged out.", LogLevel.Info);
            OnPropertyChanged(nameof(CurrentUser));
            OnPropertyChanged(nameof(IsAuthenticated));
            OnPropertyChanged(nameof(IsAdmin));
        });

        // Update Commands
        CheckUpdatesCommand = new RelayCommand(async () => await ExecuteCheckUpdatesAsync());
        ApplyUpdateCommand = new RelayCommand(async () => await ExecuteApplyUpdateAsync());
        DismissUpdateBannerCommand = new RelayCommand(() => IsUpdateBannerVisible = false);

        // Auth state listener
        AuthService.AuthStateChanged += user =>
        {
            OnPropertyChanged(nameof(CurrentUser));
            OnPropertyChanged(nameof(IsAuthenticated));
            OnPropertyChanged(nameof(IsAdmin));
            OnPropertyChanged(nameof(UserRoleDisplay));
        };

        // Update listener
        UpdateService.UpdateAvailable += update =>
        {
            IsUpdateBannerVisible = true;
            UpdateMessage = Loc.IsPortuguese
                ? $"Nova versão disponível: v{update.LatestVersion}! Clique para atualizar."
                : $"New version available: v{update.LatestVersion}! Click to update.";
        };

        // Service log subscriptions
        executor.OutputReceived += (msg, isErr) => AddLog(msg, isErr ? LogLevel.Error : LogLevel.Command);
        registryService.LogMessage += msg => AddLog(msg, LogLevel.Info);
        restoreService.LogMessage += msg => AddLog(msg, msg.Contains("Error", StringComparison.OrdinalIgnoreCase) ? LogLevel.Error : LogLevel.Info);
        debloatService.LogMessage += msg => AddLog(msg, msg.Contains("Success", StringComparison.OrdinalIgnoreCase) ? LogLevel.Success : LogLevel.Info);
        cleanerService.LogMessage += msg => AddLog(msg, msg.Contains("Finished", StringComparison.OrdinalIgnoreCase) ? LogLevel.Success : LogLevel.Info);
        boostService.LogMessage += msg => AddLog(msg, msg.Contains("Success", StringComparison.OrdinalIgnoreCase) ? LogLevel.Success : LogLevel.Info);
        gameService.LogMessage += msg => AddLog(msg, msg.Contains("Success", StringComparison.OrdinalIgnoreCase) ? LogLevel.Success : LogLevel.Info);

        AddLog("==================================================================", LogLevel.Command);
        AddLog("CAPIVARA BOOSTER - Windows 10/11 Esports & Low-Latency Tuning Suite", LogLevel.Command);
        AddLog("Privileged Administrator Elevation: ACTIVE", LogLevel.Success);
        AddLog("==================================================================", LogLevel.Command);
    }

    #region Properties

    public UserAccount? CurrentUser => AuthService.CurrentUser;
    public bool IsAuthenticated => AuthService.IsAuthenticated;
    public bool IsAdmin => AuthService.IsAdmin;

    public string UserRoleDisplay
    {
        get
        {
            if (CurrentUser == null) return "Guest";
            return CurrentUser.IsAdmin ? "ADMIN" : "USER";
        }
    }

    public NavigationTab CurrentTab
    {
        get => _currentTab;
        set
        {
            if (SetProperty(ref _currentTab, value))
            {
                OnPropertyChanged(nameof(IsDashboardActive));
                OnPropertyChanged(nameof(IsDebloatActive));
                OnPropertyChanged(nameof(IsCleanerActive));
                OnPropertyChanged(nameof(IsSystemBoostActive));
                OnPropertyChanged(nameof(IsGameOptimizerActive));
                OnPropertyChanged(nameof(IsBackupRestoreActive));

                if (_currentTab == NavigationTab.BackupRestore)
                {
                    BackupRestoreVm.Refresh();
                }
            }
        }
    }

    public bool IsDashboardActive
    {
        get => CurrentTab == NavigationTab.Dashboard;
        set
        {
            if (value && CurrentTab != NavigationTab.Dashboard)
                CurrentTab = NavigationTab.Dashboard;
        }
    }

    public bool IsDebloatActive
    {
        get => CurrentTab == NavigationTab.Debloat;
        set
        {
            if (value && CurrentTab != NavigationTab.Debloat)
                CurrentTab = NavigationTab.Debloat;
        }
    }

    public bool IsCleanerActive
    {
        get => CurrentTab == NavigationTab.Cleaner;
        set
        {
            if (value && CurrentTab != NavigationTab.Cleaner)
                CurrentTab = NavigationTab.Cleaner;
        }
    }

    public bool IsSystemBoostActive
    {
        get => CurrentTab == NavigationTab.SystemBoost;
        set
        {
            if (value && CurrentTab != NavigationTab.SystemBoost)
                CurrentTab = NavigationTab.SystemBoost;
        }
    }

    public bool IsGameOptimizerActive
    {
        get => CurrentTab == NavigationTab.GameOptimizer;
        set
        {
            if (value && CurrentTab != NavigationTab.GameOptimizer)
                CurrentTab = NavigationTab.GameOptimizer;
        }
    }

    public bool IsBackupRestoreActive
    {
        get => CurrentTab == NavigationTab.BackupRestore;
        set
        {
            if (value && CurrentTab != NavigationTab.BackupRestore)
                CurrentTab = NavigationTab.BackupRestore;
        }
    }

    public string StatusText
    {
        get => _statusText;
        set => SetProperty(ref _statusText, value);
    }

    public bool IsGlobalBusy
    {
        get => _isGlobalBusy;
        set => SetProperty(ref _isGlobalBusy, value);
    }

    public bool IsAuthModalOpen
    {
        get => _isAuthModalOpen;
        set => SetProperty(ref _isAuthModalOpen, value);
    }

    public bool IsRegisterMode
    {
        get => _isRegisterMode;
        set => SetProperty(ref _isRegisterMode, value);
    }

    public string AuthEmail
    {
        get => _authEmail;
        set => SetProperty(ref _authEmail, value);
    }

    public string AuthPassword
    {
        get => _authPassword;
        set => SetProperty(ref _authPassword, value);
    }

    public string AuthErrorMessage
    {
        get => _authErrorMessage;
        set => SetProperty(ref _authErrorMessage, value);
    }

    public bool IsUpdateBannerVisible
    {
        get => _isUpdateBannerVisible;
        set => SetProperty(ref _isUpdateBannerVisible, value);
    }

    public string UpdateMessage
    {
        get => _updateMessage;
        set => SetProperty(ref _updateMessage, value);
    }

    #endregion

    #region Commands

    public ICommand NavigateCommand { get; }
    public ICommand ClearLogsCommand { get; }
    public ICommand CopyLogsCommand { get; }
    public ICommand ChangeLanguageCommand { get; }

    public ICommand OpenAuthModalCommand { get; }
    public ICommand CloseAuthModalCommand { get; }
    public ICommand SwitchAuthModeCommand { get; }
    public ICommand LoginCommand { get; }
    public ICommand RegisterCommand { get; }
    public ICommand LogoutCommand { get; }

    public ICommand CheckUpdatesCommand { get; }
    public ICommand ApplyUpdateCommand { get; }
    public ICommand DismissUpdateBannerCommand { get; }

    #endregion

    #region Methods

    private async Task ExecuteLoginAsync()
    {
        AuthErrorMessage = string.Empty;
        var (success, msg) = await AuthService.LoginAsync(AuthEmail, AuthPassword);
        if (success)
        {
            IsAuthModalOpen = false;
            AuthPassword = string.Empty;
            AddLog($"[Auth] {msg} ({AuthService.CurrentUser?.Email})", LogLevel.Success);
            if (AuthService.IsAdmin)
            {
                AddLog("[Auth] Master Administrator role assigned.", LogLevel.Command);
            }
        }
        else
        {
            AuthErrorMessage = msg;
        }
    }

    private async Task ExecuteRegisterAsync()
    {
        AuthErrorMessage = string.Empty;
        var (success, msg) = await AuthService.RegisterAsync(AuthEmail, AuthPassword);
        if (success)
        {
            IsAuthModalOpen = false;
            AuthPassword = string.Empty;
            AddLog($"[Auth] {msg} ({AuthService.CurrentUser?.Email})", LogLevel.Success);
        }
        else
        {
            AuthErrorMessage = msg;
        }
    }

    private async Task ExecuteCheckUpdatesAsync()
    {
        AddLog(Loc.IsPortuguese ? "Verificando atualizações no GitHub..." : "Checking for updates on GitHub...", LogLevel.Info);
        var update = await UpdateService.CheckForUpdatesAsync(silent: false);
        if (update != null && update.IsUpdateAvailable)
        {
            IsUpdateBannerVisible = true;
            UpdateMessage = Loc.IsPortuguese
                ? $"Nova versão disponível: v{update.LatestVersion}!"
                : $"New version available: v{update.LatestVersion}!";
            AddLog($"[Updater] {UpdateMessage}", LogLevel.Success);
        }
        else
        {
            AddLog(Loc.IsPortuguese
                ? "Capivara Booster está na versão mais recente."
                : "Capivara Booster is up to date.", LogLevel.Success);
        }
    }

    private async Task ExecuteApplyUpdateAsync()
    {
        AddLog(Loc.IsPortuguese ? "Baixando atualização em segundo plano..." : "Downloading update in background...", LogLevel.Info);
        await UpdateService.DownloadAndApplyUpdateAsync();
    }

    public void AddLog(string message, LogLevel level = LogLevel.Info)
    {
        if (string.IsNullOrWhiteSpace(message)) return;
        _logQueue.Enqueue(new LogItem { Message = message, Level = level });
    }

    private void FlushLogQueue(object? sender, EventArgs e)
    {
        if (_logQueue.IsEmpty) return;

        int added = 0;
        while (_logQueue.TryDequeue(out var item) && added < 80)
        {
            LogItems.Add(item);
            added++;
        }

        while (LogItems.Count > 1000)
        {
            LogItems.RemoveAt(0);
        }
    }

    private void ClearLogs()
    {
        LogItems.Clear();
        AddLog(Loc.IsPortuguese ? "Console limpo." : "Console cleared.", LogLevel.Info);
    }

    private void CopyLogs()
    {
        var sb = new StringBuilder();
        foreach (var item in LogItems)
        {
            sb.AppendLine(item.FormattedMessage);
        }
        try
        {
            Clipboard.SetText(sb.ToString());
            AddLog(Loc.IsPortuguese ? "Logs copiados para a área de transferência." : "Logs copied to clipboard.", LogLevel.Success);
        }
        catch { }
    }

    #endregion
}