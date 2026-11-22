using System.IO;
using System.Runtime.InteropServices;
using CapivaraBooster.Models;

namespace CapivaraBooster.Services;

public record CleaningSummary(long TotalBytesCleaned, int FilesDeleted, int ErrorsCount);

public interface ICleanerService
{
    event Action<string>? LogMessage;

    IReadOnlyList<CleanerItem> GetCleanerItems();
    Task ScanSizesAsync(IEnumerable<CleanerItem> items);
    Task<CleaningSummary> CleanSelectedAsync(IEnumerable<CleanerItem> items);
    Task<bool> FlushDnsCacheAsync();
}

public class CleanerService : ICleanerService
{
    public event Action<string>? LogMessage;
    private readonly IProcessExecutor _executor;

    [DllImport("Shell32.dll", CharSet = CharSet.Unicode)]
    private static extern int SHEmptyRecycleBin(IntPtr hwnd, string? pszRootPath, uint dwFlags);

    private const uint SHERB_NOCONFIRMATION = 0x00000001;
    private const uint SHERB_NOPROGRESSUI = 0x00000002;
    private const uint SHERB_NOSOUND = 0x00000004;

    public CleanerService(IProcessExecutor executor)
    {
        _executor = executor;
    }

    public IReadOnlyList<CleanerItem> GetCleanerItems()
    {
        string localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        string appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
        string windir = Environment.GetFolderPath(Environment.SpecialFolder.Windows);

        return new List<CleanerItem>
        {
            new()
            {
                Category = CleanerCategory.WindowsTemp,
                DisplayName = "User Temp Files (%TEMP%)",
                Description = "Temporary files created by running user applications and installers",
                TargetPath = Path.GetTempPath().TrimEnd('\\'),
                IsSelected = true
            },
            new()
            {
                Category = CleanerCategory.SystemTemp,
                DisplayName = "Windows System Temp",
                Description = "System-level temporary files in C:\\Windows\\Temp",
                TargetPath = Path.Combine(windir, "Temp"),
                IsSelected = true
            },
            new()
            {
                Category = CleanerCategory.Prefetch,
                DisplayName = "Windows Prefetch Cache",
                Description = "Application startup caching files in C:\\Windows\\Prefetch",
                TargetPath = Path.Combine(windir, "Prefetch"),
                IsSelected = true
            },
            new()
            {
                Category = CleanerCategory.SoftwareDistribution,
                DisplayName = "Windows Update Cache",
                Description = "Downloaded installation packages in C:\\Windows\\SoftwareDistribution\\Download",
                TargetPath = Path.Combine(windir, "SoftwareDistribution", "Download"),
                IsSelected = true
            },
            new()
            {
                Category = CleanerCategory.RecycleBin,
                DisplayName = "Recycle Bin",
                Description = "Files discarded into the Windows Recycle Bin",
                TargetPath = "RecycleBin",
                IsSelected = true
            },
            new()
            {
                Category = CleanerCategory.DnsCache,
                DisplayName = "DNS Resolver Cache",
                Description = "Flushes stale IP/DNS lookups and resets network name cache",
                TargetPath = "ipconfig /flushdns",
                IsSelected = true
            },
            new()
            {
                Category = CleanerCategory.DirectXShader,
                DisplayName = "DirectX Shader Cache",
                Description = "Compiled DirectX shader cache files in D3DSCache",
                TargetPath = Path.Combine(localAppData, "D3DSCache"),
                IsSelected = true
            },
            new()
            {
                Category = CleanerCategory.NvidiaShader,
                DisplayName = "NVIDIA DX/GL Shader Cache",
                Description = "Stale NVIDIA DirectX, OpenGL, and Compute shader caches",
                TargetPath = Path.Combine(localAppData, "NVIDIA", "DXCache"),
                IsSelected = true
            },
            new()
            {
                Category = CleanerCategory.AmdShader,
                DisplayName = "AMD DxCache",
                Description = "Compiled AMD Radeon shader pipeline cache files",
                TargetPath = Path.Combine(localAppData, "AMD", "DxCache"),
                IsSelected = true
            }
        };
    }

    public async Task ScanSizesAsync(IEnumerable<CleanerItem> items)
    {
        await Task.Run(() =>
        {
            foreach (var item in items)
            {
                item.IsScanning = true;
                try
                {
                    if (item.Category == CleanerCategory.DnsCache)
                    {
                        item.SizeBytes = 0;
                        continue;
                    }

                    if (item.Category == CleanerCategory.NvidiaShader)
                    {
                        string localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
                        string appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
                        long total = CalculateDirectorySize(Path.Combine(localAppData, "NVIDIA", "DXCache"))
                                   + CalculateDirectorySize(Path.Combine(localAppData, "NVIDIA", "GLCache"))
                                   + CalculateDirectorySize(Path.Combine(appData, "NVIDIA", "ComputeCache"));
                        item.SizeBytes = total;
                        continue;
                    }

                    if (item.Category == CleanerCategory.AmdShader)
                    {
                        string localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
                        long total = CalculateDirectorySize(Path.Combine(localAppData, "AMD", "DxCache"))
                                   + CalculateDirectorySize(Path.Combine(localAppData, "AMD", "GLCache"));
                        item.SizeBytes = total;
                        continue;
                    }

                    if (Directory.Exists(item.TargetPath))
                    {
                        item.SizeBytes = CalculateDirectorySize(item.TargetPath);
                    }
                    else
                    {
                        item.SizeBytes = 0;
                    }
                }
                catch (Exception ex)
                {
                    LogMessage?.Invoke($"[Scan Warning] Error reading {item.DisplayName}: {ex.Message}");
                    item.SizeBytes = 0;
                }
                finally
                {
                    item.IsScanning = false;
                }
            }
        });
    }

    public async Task<CleaningSummary> CleanSelectedAsync(IEnumerable<CleanerItem> items)
    {
        return await Task.Run(async () =>
        {
            long cleanedBytes = 0;
            int filesDeleted = 0;
            int errorsCount = 0;

            foreach (var item in items.Where(x => x.IsSelected))
            {
                LogMessage?.Invoke($"[Cleaner] Processing: {item.DisplayName}...");

                try
                {
                    switch (item.Category)
                    {
                        case CleanerCategory.DnsCache:
                            await FlushDnsCacheAsync();
                            item.IsCleaned = true;
                            break;

                        case CleanerCategory.RecycleBin:
                            int hr = SHEmptyRecycleBin(IntPtr.Zero, null, SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND);
                            LogMessage?.Invoke($"[Cleaner] Recycle Bin emptied (Result code: 0x{hr:X8})");
                            item.IsCleaned = true;
                            break;

                        case CleanerCategory.SoftwareDistribution:
                            await _executor.ExecuteCommandAsync("net stop wuauserv");
                            var sdResult = CleanDirectoryContents(item.TargetPath);
                            cleanedBytes += sdResult.Bytes;
                            filesDeleted += sdResult.Count;
                            errorsCount += sdResult.Errors;
                            await _executor.ExecuteCommandAsync("net start wuauserv");
                            item.IsCleaned = true;
                            break;

                        case CleanerCategory.NvidiaShader:
                            string localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
                            string appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
                            var nv1 = CleanDirectoryContents(Path.Combine(localAppData, "NVIDIA", "DXCache"));
                            var nv2 = CleanDirectoryContents(Path.Combine(localAppData, "NVIDIA", "GLCache"));
                            var nv3 = CleanDirectoryContents(Path.Combine(appData, "NVIDIA", "ComputeCache"));
                            cleanedBytes += nv1.Bytes + nv2.Bytes + nv3.Bytes;
                            filesDeleted += nv1.Count + nv2.Count + nv3.Count;
                            item.IsCleaned = true;
                            break;

                        case CleanerCategory.AmdShader:
                            string amdLocal = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
                            var amd1 = CleanDirectoryContents(Path.Combine(amdLocal, "AMD", "DxCache"));
                            var amd2 = CleanDirectoryContents(Path.Combine(amdLocal, "AMD", "GLCache"));
                            cleanedBytes += amd1.Bytes + amd2.Bytes;
                            filesDeleted += amd1.Count + amd2.Count;
                            item.IsCleaned = true;
                            break;

                        default:
                            if (Directory.Exists(item.TargetPath))
                            {
                                var dirResult = CleanDirectoryContents(item.TargetPath);
                                cleanedBytes += dirResult.Bytes;
                                filesDeleted += dirResult.Count;
                                errorsCount += dirResult.Errors;
                            }
                            item.IsCleaned = true;
                            break;
                    }
                }
                catch (Exception ex)
                {
                    errorsCount++;
                    LogMessage?.Invoke($"[Cleaner Error] Error cleaning {item.DisplayName}: {ex.Message}");
                }
            }

            LogMessage?.Invoke($"[Cleaner Finished] Successfully purged {filesDeleted} junk files ({FormatBytes(cleanedBytes)}).");
            return new CleaningSummary(cleanedBytes, filesDeleted, errorsCount);
        });
    }

    public async Task<bool> FlushDnsCacheAsync()
    {
        LogMessage?.Invoke("[Cleaner] Flushing Windows DNS Resolver Cache...");
        var result = await _executor.ExecuteCommandAsync("ipconfig /flushdns");
        return result.IsSuccess;
    }

    private (long Bytes, int Count, int Errors) CleanDirectoryContents(string path)
    {
        if (!Directory.Exists(path)) return (0, 0, 0);

        long bytes = 0;
        int count = 0;
        int errors = 0;

        try
        {
            var dirInfo = new DirectoryInfo(path);

            foreach (var file in dirInfo.EnumerateFiles("*", SearchOption.AllDirectories))
            {
                try
                {
                    long len = file.Length;
                    file.Attributes = FileAttributes.Normal;
                    file.Delete();
                    bytes += len;
                    count++;
                }
                catch
                {
                    errors++;
                }
            }

            foreach (var subDir in dirInfo.EnumerateDirectories())
            {
                try
                {
                    subDir.Delete(true);
                }
                catch
                {
                    errors++;
                }
            }
        }
        catch (Exception ex)
        {
            LogMessage?.Invoke($"[Cleaner Notice] Skipping directory {path}: {ex.Message}");
        }

        return (bytes, count, errors);
    }

    private long CalculateDirectorySize(string path)
    {
        if (!Directory.Exists(path)) return 0;

        long size = 0;
        try
        {
            var dir = new DirectoryInfo(path);
            foreach (var file in dir.EnumerateFiles("*", SearchOption.AllDirectories))
            {
                try { size += file.Length; } catch { }
            }
        }
        catch { }

        return size;
    }

    private string FormatBytes(long bytes)
    {
        if (bytes < 1024) return $"{bytes} B";
        if (bytes < 1024 * 1024) return $"{bytes / 1024.0:F1} KB";
        if (bytes < 1024 * 1024 * 1024) return $"{bytes / (1024.0 * 1024.0):F1} MB";
        return $"{bytes / (1024.0 * 1024.0 * 1024.0):F2} GB";
    }
}
