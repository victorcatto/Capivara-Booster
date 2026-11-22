using System.IO;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.RegularExpressions;
using CapivaraBooster.Models;
using Microsoft.Win32;

namespace CapivaraBooster.Services;

public record GameDetectionResult(
    string GameId,
    bool Found,
    string ExecutablePath,
    string InstallDirectory
);

public interface IGameOptimizerService
{
    event Action<string>? LogMessage;

    Task<GameDetectionResult> DetectGameAsync(string gameId);
    Task<bool> ApplyProcessPriorityAndAffinityAsync(string processName, bool highPriority, bool disableCore0);
    Task<bool> ApplyAppCompatFlagsAsync(string exePath, bool disableFse, bool highDpi);

    // CS2
    Task<bool> OptimizeCs2Async(string cs2ExePath, bool generateAutoexec, bool clearShaderCache);
    string GetCs2RecommendedLaunchOptions();

    // Valorant
    Task<bool> OptimizeValorantAsync(string valorantExePath, bool setRiotClientLowPriority, bool clearLogs, bool addDefenderExclusions);

    // League of Legends
    Task<bool> OptimizeLeagueOfLegendsAsync(string lolExePath, bool enableLowSpecAndCloseClient, bool clearLogs, bool addDefenderExclusions);

    Task<bool> RevertGameTweaksAsync(string gameId, string exePath);
}

public class GameOptimizerService : IGameOptimizerService
{
    public event Action<string>? LogMessage;
    private readonly IProcessExecutor _executor;
    private readonly IRegistryService _registryService;
    private readonly IRestoreService _restoreService;

    public GameOptimizerService(IProcessExecutor executor, IRegistryService registryService, IRestoreService restoreService)
    {
        _executor = executor;
        _registryService = registryService;
        _restoreService = restoreService;
    }

    public async Task<GameDetectionResult> DetectGameAsync(string gameId)
    {
        return await Task.Run(() =>
        {
            switch (gameId.ToLowerInvariant())
            {
                case "cs2":
                    return DetectCs2();
                case "valorant":
                    return DetectValorant();
                case "lol":
                    return DetectLeagueOfLegends();
                default:
                    return new GameDetectionResult(gameId, false, string.Empty, string.Empty);
            }
        });
    }

    #region Detection Implementations

    private GameDetectionResult DetectCs2()
    {
        string? steamPath = _registryService.GetString(RegistryHive.CurrentUser, @"Software\Valve\Steam", "SteamPath");
        if (string.IsNullOrEmpty(steamPath))
        {
            steamPath = _registryService.GetString(RegistryHive.LocalMachine, @"SOFTWARE\Valve\Steam", "InstallPath");
        }

        var candidateLibraries = new List<string>();
        if (!string.IsNullOrEmpty(steamPath) && Directory.Exists(steamPath))
        {
            candidateLibraries.Add(steamPath);

            string vdfPath = Path.Combine(steamPath, "steamapps", "libraryfolders.vdf");
            if (File.Exists(vdfPath))
            {
                try
                {
                    string content = File.ReadAllText(vdfPath);
                    var matches = Regex.Matches(content, @"""path""\s+""([^""]+)""", RegexOptions.IgnoreCase);
                    foreach (Match m in matches)
                    {
                        string p = m.Groups[1].Value.Replace(@"\\", @"\");
                        if (Directory.Exists(p) && !candidateLibraries.Contains(p, StringComparer.OrdinalIgnoreCase))
                        {
                            candidateLibraries.Add(p);
                        }
                    }
                }
                catch { }
            }
        }

        string[] standardPaths = { @"C:\Program Files (x86)\Steam", @"D:\SteamLibrary", @"E:\SteamLibrary", @"F:\SteamLibrary" };
        foreach (var p in standardPaths)
        {
            if (Directory.Exists(p) && !candidateLibraries.Contains(p, StringComparer.OrdinalIgnoreCase))
            {
                candidateLibraries.Add(p);
            }
        }

        foreach (var lib in candidateLibraries)
        {
            string cs2Exe = Path.Combine(lib, "steamapps", "common", "Counter-Strike Global Offensive", "game", "bin", "win64", "cs2.exe");
            if (File.Exists(cs2Exe))
            {
                LogMessage?.Invoke($"[Detection] Found Counter-Strike 2 at: {cs2Exe}");
                return new GameDetectionResult("cs2", true, cs2Exe, Path.GetDirectoryName(cs2Exe)!);
            }
        }

        LogMessage?.Invoke("[Detection] CS2 executable not found in common Steam libraries.");
        return new GameDetectionResult("cs2", false, string.Empty, string.Empty);
    }

    private GameDetectionResult DetectValorant()
    {
        string riotJson = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData), "Riot Games", "RiotClientInstalls.json");
        if (File.Exists(riotJson))
        {
            try
            {
                string json = File.ReadAllText(riotJson);
                using var doc = JsonDocument.Parse(json);
                if (doc.RootElement.TryGetProperty("rc_live", out var liveProp))
                {
                    string? riotLive = liveProp.GetString();
                    if (!string.IsNullOrEmpty(riotLive))
                    {
                        string valRoot = Path.Combine(Path.GetDirectoryName(riotLive)!, "..", "VALORANT", "live", "ShooterGame", "Binaries", "Win64", "VALORANT-Win64-Shipping.exe");
                        valRoot = Path.GetFullPath(valRoot);
                        if (File.Exists(valRoot))
                        {
                            LogMessage?.Invoke($"[Detection] Found Valorant at: {valRoot}");
                            return new GameDetectionResult("valorant", true, valRoot, Path.GetDirectoryName(valRoot)!);
                        }
                    }
                }
            }
            catch { }
        }

        string[] candidates =
        {
            @"C:\Riot Games\VALORANT\live\ShooterGame\Binaries\Win64\VALORANT-Win64-Shipping.exe",
            @"D:\Riot Games\VALORANT\live\ShooterGame\Binaries\Win64\VALORANT-Win64-Shipping.exe",
            @"E:\Riot Games\VALORANT\live\ShooterGame\Binaries\Win64\VALORANT-Win64-Shipping.exe"
        };

        foreach (var path in candidates)
        {
            if (File.Exists(path))
            {
                LogMessage?.Invoke($"[Detection] Found Valorant at: {path}");
                return new GameDetectionResult("valorant", true, path, Path.GetDirectoryName(path)!);
            }
        }

        LogMessage?.Invoke("[Detection] Valorant executable not found in standard Riot Games locations.");
        return new GameDetectionResult("valorant", false, string.Empty, string.Empty);
    }

    private GameDetectionResult DetectLeagueOfLegends()
    {
        string riotJson = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData), "Riot Games", "RiotClientInstalls.json");
        if (File.Exists(riotJson))
        {
            try
            {
                string json = File.ReadAllText(riotJson);
                using var doc = JsonDocument.Parse(json);
                if (doc.RootElement.TryGetProperty("league_of_legends.live", out var liveProp))
                {
                    string? lolPath = liveProp.GetString();
                    if (!string.IsNullOrEmpty(lolPath))
                    {
                        string exe = Path.Combine(lolPath, "League of Legends.exe");
                        if (File.Exists(exe))
                        {
                            LogMessage?.Invoke($"[Detection] Found League of Legends at: {exe}");
                            return new GameDetectionResult("lol", true, exe, lolPath);
                        }
                    }
                }
            }
            catch { }
        }

        string[] candidates =
        {
            @"C:\Riot Games\League of Legends\League of Legends.exe",
            @"D:\Riot Games\League of Legends\League of Legends.exe",
            @"E:\Riot Games\League of Legends\League of Legends.exe"
        };

        foreach (var path in candidates)
        {
            if (File.Exists(path))
            {
                LogMessage?.Invoke($"[Detection] Found League of Legends at: {path}");
                return new GameDetectionResult("lol", true, path, Path.GetDirectoryName(path)!);
            }
        }

        LogMessage?.Invoke("[Detection] League of Legends executable not found in standard Riot Games locations.");
        return new GameDetectionResult("lol", false, string.Empty, string.Empty);
    }

    #endregion

    #region Priority, Affinity & AppCompatFlags

    public async Task<bool> ApplyProcessPriorityAndAffinityAsync(string processName, bool highPriority, bool disableCore0)
    {
        return await Task.Run(() =>
        {
            string ifeoKey = $@"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\{processName}\PerfOptions";

            if (highPriority)
            {
                _registryService.SetDword(RegistryHive.LocalMachine, ifeoKey, "CpuPriorityClass", 3);
                LogMessage?.Invoke($"[IFEO] Set CPU Priority for '{processName}' to High (Class 3).");
            }

            if (disableCore0)
            {
                _registryService.SetDword(RegistryHive.LocalMachine, ifeoKey, "CpuAffinityMask", unchecked((int)0xFFFFFFFE));
                LogMessage?.Invoke($"[IFEO] Set CPU Affinity Mask for '{processName}' to omit Core 0 (0xFFFFFFFE).");
            }

            return true;
        });
    }

    public async Task<bool> ApplyAppCompatFlagsAsync(string exePath, bool disableFse, bool highDpi)
    {
        return await Task.Run(() =>
        {
            if (!File.Exists(exePath))
            {
                LogMessage?.Invoke($"[AppCompat] File does not exist: {exePath}");
                return false;
            }

            string flags = "~";
            if (disableFse) flags += " DISABLEDXMAXIMIZEDWINDOWEDMODE";
            if (highDpi) flags += " HIGHDPIAWARE";

            string layersKey = @"Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers";
            _registryService.SetString(RegistryHive.CurrentUser, layersKey, exePath, flags.Trim());

            LogMessage?.Invoke($"[AppCompat] Applied AppCompatFlags for '{Path.GetFileName(exePath)}': {flags.Trim()}");
            return true;
        });
    }

    #endregion

    #region Counter-Strike 2

    public async Task<bool> OptimizeCs2Async(string cs2ExePath, bool generateAutoexec, bool clearShaderCache)
    {
        LogMessage?.Invoke("[CS2] Applying CS2 optimizations...");

        await ApplyProcessPriorityAndAffinityAsync("cs2.exe", highPriority: true, disableCore0: true);

        if (File.Exists(cs2ExePath))
        {
            await ApplyAppCompatFlagsAsync(cs2ExePath, disableFse: true, highDpi: true);
        }

        if (clearShaderCache && File.Exists(cs2ExePath))
        {
            try
            {
                string? gameDir = Path.GetDirectoryName(cs2ExePath);
                if (gameDir != null)
                {
                    string steamAppsDir = Path.GetFullPath(Path.Combine(gameDir, @"..\..\..\..\"));
                    string shaderCache730 = Path.Combine(steamAppsDir, "shadercache", "730");
                    if (Directory.Exists(shaderCache730))
                    {
                        Directory.Delete(shaderCache730, true);
                        LogMessage?.Invoke($"[CS2 Cache] Cleared CS2 Shader Cache at: {shaderCache730}");
                    }
                }
            }
            catch (Exception ex)
            {
                LogMessage?.Invoke($"[CS2 Cache Warning] Notice while clearing shader cache: {ex.Message}");
            }
        }

        if (generateAutoexec && File.Exists(cs2ExePath))
        {
            try
            {
                string? gameBinDir = Path.GetDirectoryName(cs2ExePath);
                if (gameBinDir != null)
                {
                    string cfgDir = Path.GetFullPath(Path.Combine(gameBinDir, @"..\..\csgo\cfg"));
                    if (!Directory.Exists(cfgDir)) Directory.CreateDirectory(cfgDir);

                    string autoexecPath = Path.Combine(cfgDir, "autoexec.cfg");
                    if (File.Exists(autoexecPath))
                    {
                        await _restoreService.BackupFileAsync(autoexecPath, "CS2", "Original CS2 autoexec.cfg");
                    }

                    string autoexecContent = GenerateCs2AutoexecContent();
                    await File.WriteAllTextAsync(autoexecPath, autoexecContent, Encoding.UTF8);
                    LogMessage?.Invoke($"[CS2 Autoexec] Successfully created optimized autoexec.cfg at: {autoexecPath}");
                }
            }
            catch (Exception ex)
            {
                LogMessage?.Invoke($"[CS2 Autoexec Error] Failed to generate autoexec: {ex.Message}");
            }
        }

        LogMessage?.Invoke("[CS2] CS2 optimization complete.");
        return true;
    }

    public string GetCs2RecommendedLaunchOptions()
    {
        return "-high -novid +fps_max 0 -nojoy +exec autoexec.cfg";
    }

    private string GenerateCs2AutoexecContent()
    {
        var sb = new StringBuilder();
        sb.AppendLine("// ==========================================================================");
        sb.AppendLine("// Capivara Booster - Counter-Strike 2 Esports Competitive Config");
        sb.AppendLine($"// Generated at: {DateTime.Now:yyyy-MM-dd HH:mm:ss}");
        sb.AppendLine("// ==========================================================================");
        sb.AppendLine();
        sb.AppendLine("// [NETWORK, TICKRATE & INTERPOLATION]");
        sb.AppendLine("rate \"786432\"");
        sb.AppendLine("cl_net_buffer_ticks \"0\"");
        sb.AppendLine("cl_updaterate \"128\"");
        sb.AppendLine("cl_interp \"0.015625\"");
        sb.AppendLine("cl_interp_ratio \"1\"");
        sb.AppendLine();
        sb.AppendLine("// [PERFORMANCE & LOW LATENCY]");
        sb.AppendLine("fps_max \"0\"");
        sb.AppendLine("fps_max_ui \"120\"");
        sb.AppendLine("engine_low_latency_sleep_after_client_tick \"true\"");
        sb.AppendLine("r_drawtracers_firstperson \"0\"");
        sb.AppendLine("r_show_build_info \"false\"");
        sb.AppendLine("cl_autohelp \"false\"");
        sb.AppendLine("gameinstructor_enable \"false\"");
        sb.AppendLine();
        sb.AppendLine("// [AUDIO LATENCY & SPATIALIZATION]");
        sb.AppendLine("snd_headphone_eq \"0\"");
        sb.AppendLine("snd_spatialize_roundoff \"1\"");
        sb.AppendLine("snd_mixahead \"0.011\"");
        sb.AppendLine();
        sb.AppendLine("// [DE-SUBTICK MOVEMENT & JUMPTHROW]");
        sb.AppendLine("alias \"+jumpaction\" \"+jump;\"");
        sb.AppendLine("alias \"-jumpaction\" \"-jump;\"");
        sb.AppendLine("alias \"+throwaction\" \"-attack; -attack2;\"");
        sb.AppendLine("alias \"+jumpthrow\" \"+jumpaction; +throwaction;\"");
        sb.AppendLine("alias \"-jumpthrow\" \"-jumpaction;\"");
        sb.AppendLine("bind \"v\" \"+jumpthrow\"");
        sb.AppendLine();
        sb.AppendLine("// [MOUSE & INPUT CLARITY]");
        sb.AppendLine("m_pitch \"0.022\"");
        sb.AppendLine("m_yaw \"0.022\"");
        sb.AppendLine();
        sb.AppendLine("host_writeconfig");
        sb.AppendLine("echo \"[CapivaraBooster] CS2 autoexec loaded successfully!\"");
        return sb.ToString();
    }

    #endregion

    #region Valorant

    public async Task<bool> OptimizeValorantAsync(string valorantExePath, bool setRiotClientLowPriority, bool clearLogs, bool addDefenderExclusions)
    {
        LogMessage?.Invoke("[Valorant] Applying Valorant & Riot Vanguard optimizations...");

        await ApplyProcessPriorityAndAffinityAsync("VALORANT-Win64-Shipping.exe", highPriority: true, disableCore0: true);

        if (setRiotClientLowPriority)
        {
            string ifeoClient = @"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\RiotClientServices.exe\PerfOptions";
            _registryService.SetDword(RegistryHive.LocalMachine, ifeoClient, "CpuPriorityClass", 1);
            LogMessage?.Invoke("[Valorant] RiotClientServices.exe priority set to Low (Idle) to prevent background micro-stutters.");
        }

        if (File.Exists(valorantExePath))
        {
            await ApplyAppCompatFlagsAsync(valorantExePath, disableFse: true, highDpi: true);
        }

        if (clearLogs)
        {
            try
            {
                string localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
                string[] valLogPaths =
                {
                    Path.Combine(localAppData, "VALORANT", "Saved", "Logs"),
                    Path.Combine(localAppData, "VALORANT", "Saved", "Crashes"),
                    Path.Combine(localAppData, "VALORANT", "Saved", "webcache"),
                    Path.Combine(localAppData, "Riot Games", "Riot Client", "Logs")
                };

                foreach (var logDir in valLogPaths)
                {
                    if (Directory.Exists(logDir))
                    {
                        Directory.Delete(logDir, true);
                        LogMessage?.Invoke($"[Valorant Logs] Cleared directory: {logDir}");
                    }
                }
            }
            catch (Exception ex)
            {
                LogMessage?.Invoke($"[Valorant Warning] Notice while clearing logs: {ex.Message}");
            }
        }

        if (addDefenderExclusions)
        {
            LogMessage?.Invoke("[Valorant] Adding Windows Defender exclusions for Riot Vanguard and Valorant...");

            string vanguardPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles), "Riot Vanguard");
            string riotGamesDir = Path.GetFullPath(Path.Combine(Path.GetDirectoryName(valorantExePath) ?? @"C:\Riot Games\VALORANT", @"..\..\..\..\"));

            string psScript = $@"
Add-MpPreference -ExclusionPath '{vanguardPath}' -ErrorAction SilentlyContinue
Add-MpPreference -ExclusionPath '{riotGamesDir}' -ErrorAction SilentlyContinue
Add-MpPreference -ExclusionProcess 'vgk.sys' -ErrorAction SilentlyContinue
Add-MpPreference -ExclusionProcess 'vgc.exe' -ErrorAction SilentlyContinue
Add-MpPreference -ExclusionProcess 'VALORANT-Win64-Shipping.exe' -ErrorAction SilentlyContinue
Add-MpPreference -ExclusionProcess 'RiotClientServices.exe' -ErrorAction SilentlyContinue
";
            await _executor.ExecutePowerShellAsync(psScript);
            LogMessage?.Invoke("[Valorant Defender] Added exclusions for vgk.sys, vgc.exe, and Riot installation paths.");
        }

        LogMessage?.Invoke("[Valorant] Valorant optimization complete.");
        return true;
    }

    #endregion

    #region League of Legends

    public async Task<bool> OptimizeLeagueOfLegendsAsync(string lolExePath, bool enableLowSpecAndCloseClient, bool clearLogs, bool addDefenderExclusions)
    {
        LogMessage?.Invoke("[LoL] Applying League of Legends optimizations...");

        await ApplyProcessPriorityAndAffinityAsync("League of Legends.exe", highPriority: true, disableCore0: false);

        if (File.Exists(lolExePath))
        {
            await ApplyAppCompatFlagsAsync(lolExePath, disableFse: true, highDpi: true);
        }

        string? lolRootDir = Path.GetDirectoryName(lolExePath);
        if (lolRootDir != null && Directory.Exists(lolRootDir))
        {
            if (enableLowSpecAndCloseClient)
            {
                string configDir = Path.Combine(lolRootDir, "Config");
                if (Directory.Exists(configDir))
                {
                    string gameCfg = Path.Combine(configDir, "game.cfg");
                    string persistedSettings = Path.Combine(configDir, "PersistedSettings.json");

                    if (File.Exists(gameCfg))
                    {
                        await _restoreService.BackupFileAsync(gameCfg, "LoL", "Original LoL game.cfg");
                        PatchGameCfg(gameCfg);
                    }

                    if (File.Exists(persistedSettings))
                    {
                        await _restoreService.BackupFileAsync(persistedSettings, "LoL", "Original LoL PersistedSettings.json");
                        PatchPersistedSettingsJson(persistedSettings);
                    }
                }
            }

            if (clearLogs)
            {
                try
                {
                    string logsDir = Path.Combine(lolRootDir, "Logs");
                    if (Directory.Exists(logsDir))
                    {
                        Directory.Delete(logsDir, true);
                        LogMessage?.Invoke($"[LoL Logs] Cleared LoL game logs at: {logsDir}");
                    }

                    string localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
                    string riotLogs = Path.Combine(localAppData, "Riot Games", "League of Legends", "Logs");
                    if (Directory.Exists(riotLogs))
                    {
                        Directory.Delete(riotLogs, true);
                        LogMessage?.Invoke($"[LoL Logs] Cleared Riot client LoL logs at: {riotLogs}");
                    }
                }
                catch (Exception ex)
                {
                    LogMessage?.Invoke($"[LoL Warning] Notice while clearing logs: {ex.Message}");
                }
            }

            if (addDefenderExclusions)
            {
                LogMessage?.Invoke("[LoL] Adding Windows Defender exclusions for League of Legends...");
                string psScript = $@"
Add-MpPreference -ExclusionPath '{lolRootDir}' -ErrorAction SilentlyContinue
Add-MpPreference -ExclusionProcess 'League of Legends.exe' -ErrorAction SilentlyContinue
Add-MpPreference -ExclusionProcess 'LeagueClient.exe' -ErrorAction SilentlyContinue
Add-MpPreference -ExclusionProcess 'LeagueClientUx.exe' -ErrorAction SilentlyContinue
";
                await _executor.ExecutePowerShellAsync(psScript);
                LogMessage?.Invoke("[LoL Defender] Added exclusions for League of Legends folder and executables.");
            }
        }

        LogMessage?.Invoke("[LoL] League of Legends optimization complete.");
        return true;
    }

    private void PatchGameCfg(string path)
    {
        try
        {
            var lines = File.ReadAllLines(path).ToList();
            bool inGeneral = false;
            bool foundLowSpec = false;
            bool foundCloseClient = false;
            bool foundVsync = false;

            for (int i = 0; i < lines.Count; i++)
            {
                string line = lines[i].Trim();
                if (line.StartsWith("[General]", StringComparison.OrdinalIgnoreCase))
                {
                    inGeneral = true;
                    continue;
                }
                else if (line.StartsWith("[") && line.EndsWith("]"))
                {
                    if (inGeneral)
                    {
                        if (!foundLowSpec) lines.Insert(i++, "LowSpecMode=1");
                        if (!foundCloseClient) lines.Insert(i++, "CloseClient=1");
                        if (!foundVsync) lines.Insert(i++, "WaitForVerticalSync=0");
                        inGeneral = false;
                    }
                }

                if (inGeneral)
                {
                    if (line.StartsWith("LowSpecMode", StringComparison.OrdinalIgnoreCase))
                    {
                        lines[i] = "LowSpecMode=1";
                        foundLowSpec = true;
                    }
                    else if (line.StartsWith("CloseClient", StringComparison.OrdinalIgnoreCase))
                    {
                        lines[i] = "CloseClient=1";
                        foundCloseClient = true;
                    }
                    else if (line.StartsWith("WaitForVerticalSync", StringComparison.OrdinalIgnoreCase))
                    {
                        lines[i] = "WaitForVerticalSync=0";
                        foundVsync = true;
                    }
                }
            }

            if (inGeneral)
            {
                if (!foundLowSpec) lines.Add("LowSpecMode=1");
                if (!foundCloseClient) lines.Add("CloseClient=1");
                if (!foundVsync) lines.Add("WaitForVerticalSync=0");
            }

            File.WriteAllLines(path, lines);
            LogMessage?.Invoke("[LoL] Successfully configured Low Spec Mode and Close Client in game.cfg.");
        }
        catch (Exception ex)
        {
            LogMessage?.Invoke($"[LoL Error] Failed to update game.cfg: {ex.Message}");
        }
    }

    private void PatchPersistedSettingsJson(string path)
    {
        try
        {
            string json = File.ReadAllText(path);
            var node = JsonNode.Parse(json);
            if (node is JsonObject rootObj && rootObj.TryGetPropertyValue("files", out var filesNode) && filesNode is JsonArray filesArray)
            {
                foreach (var fileElem in filesArray)
                {
                    if (fileElem is JsonObject fileObj && fileObj.TryGetPropertyValue("sections", out var sectionsNode) && sectionsNode is JsonArray sectionsArray)
                    {
                        foreach (var secElem in sectionsArray)
                        {
                            if (secElem is JsonObject secObj && secObj.TryGetPropertyValue("name", out var secName) && secName?.ToString() == "General")
                            {
                                if (secObj.TryGetPropertyValue("settings", out var settingsNode) && settingsNode is JsonArray settingsArray)
                                {
                                    UpdateOrAddSetting(settingsArray, "LowSpecMode", "1");
                                    UpdateOrAddSetting(settingsArray, "CloseClientDuringGame", "1");
                                    UpdateOrAddSetting(settingsArray, "WaitForVerticalSync", "0");
                                }
                            }
                        }
                    }
                }

                var options = new JsonSerializerOptions { WriteIndented = true };
                File.WriteAllText(path, node.ToJsonString(options));
                LogMessage?.Invoke("[LoL] Successfully updated PersistedSettings.json with LowSpecMode=1 & CloseClientDuringGame=1.");
            }
        }
        catch (Exception ex)
        {
            LogMessage?.Invoke($"[LoL Error] Notice while patching PersistedSettings.json: {ex.Message}");
        }
    }

    private void UpdateOrAddSetting(JsonArray settingsArray, string name, string val)
    {
        foreach (var item in settingsArray)
        {
            if (item is JsonObject obj && obj.TryGetPropertyValue("name", out var n) && n?.ToString() == name)
            {
                obj["value"] = val;
                return;
            }
        }

        var newSetting = new JsonObject
        {
            ["name"] = name,
            ["value"] = val
        };
        settingsArray.Add(newSetting);
    }

    #endregion

    #region Revert

    public async Task<bool> RevertGameTweaksAsync(string gameId, string exePath)
    {
        return await Task.Run(() =>
        {
            LogMessage?.Invoke($"[Revert] Reverting tweaks for {gameId}...");

            string exeName = Path.GetFileName(exePath);
            if (string.IsNullOrEmpty(exeName))
            {
                exeName = gameId switch
                {
                    "cs2" => "cs2.exe",
                    "valorant" => "VALORANT-Win64-Shipping.exe",
                    "lol" => "League of Legends.exe",
                    _ => ""
                };
            }

            if (!string.IsNullOrEmpty(exeName))
            {
                string ifeoKey = $@"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\{exeName}\PerfOptions";
                _registryService.DeleteKeyTree(RegistryHive.LocalMachine, ifeoKey);
            }

            if (gameId == "valorant")
            {
                string ifeoClient = @"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\RiotClientServices.exe\PerfOptions";
                _registryService.DeleteKeyTree(RegistryHive.LocalMachine, ifeoClient);
            }

            if (!string.IsNullOrEmpty(exePath))
            {
                string layersKey = @"Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers";
                _registryService.DeleteValue(RegistryHive.CurrentUser, layersKey, exePath);
            }

            LogMessage?.Invoke($"[Revert] Tweaks for {gameId} successfully restored to defaults.");
            return true;
        });
    }

    #endregion
}
