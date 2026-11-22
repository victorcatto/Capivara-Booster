using System.Windows;
using CapivaraBooster.Services;
using CapivaraBooster.ViewModels;

namespace CapivaraBooster;

public partial class App : Application
{
    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

        AppDomain.CurrentDomain.UnhandledException += (s, args) =>
        {
            MessageBox.Show($"Capivara Booster Critical Error: {args.ExceptionObject}", "Capivara Booster", MessageBoxButton.OK, MessageBoxImage.Error);
        };

        DispatcherUnhandledException += (s, args) =>
        {
            MessageBox.Show($"Capivara Booster Warning: {args.Exception.Message}", "Capivara Booster", MessageBoxButton.OK, MessageBoxImage.Warning);
            args.Handled = true;
        };

        // 1. Initialize Core Services
        var loc = LocalizationService.Instance;
        var executor = new ProcessExecutor();
        var registry = new RegistryService();
        var restore = new RestoreService(executor, registry);
        var debloat = new DebloatService(executor, registry);
        var cleaner = new CleanerService(executor);
        var boost = new SystemBoostService(executor, registry);
        var game = new GameOptimizerService(executor, registry, restore);
        var auth = new AuthService(loc);
        var updater = new UpdateService(loc);

        // 2. Initialize ViewModels
        var dashboardVm = new DashboardViewModel(boost, cleaner, restore, game);
        var debloatVm = new DebloatViewModel(debloat, restore);
        var cleanerVm = new CleanerViewModel(cleaner);
        var systemBoostVm = new SystemBoostViewModel(boost, restore);
        var gameOptimizerVm = new GameOptimizerViewModel(game, restore);
        var backupRestoreVm = new BackupRestoreViewModel(restore, registry);

        var mainVm = new MainViewModel(
            dashboardVm,
            debloatVm,
            cleanerVm,
            systemBoostVm,
            gameOptimizerVm,
            backupRestoreVm,
            executor,
            registry,
            restore,
            debloat,
            cleaner,
            boost,
            game,
            auth,
            updater
        );

        // 3. Show MainWindow
        var mainWindow = new MainWindow(mainVm);
        mainWindow.Show();

        // 4. Background Initialization (Optimized Concurrent Initialization)
        _ = Task.Run(async () =>
        {
            try
            {
                // Run status refresh, boost init, and update check concurrently
                var statusTask = dashboardVm.RefreshStatusAsync();
                var boostTask = systemBoostVm.InitializeAsync();
                var updateTask = updater.CheckForUpdatesAsync(silent: true);

                await Task.WhenAll(statusTask, boostTask, updateTask);

                // Run cleaner size scan in background without blocking startup
                await cleanerVm.ScanAsync();
            }
            catch { }
        });
    }
}
