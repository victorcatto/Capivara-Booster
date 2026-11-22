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
                Description = "Compiled DirectX shader cache files (D3DSCache & DirectX Shader Cache)",
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
        // High-performance asynchronous parallel directory scanning
        await Parallel.ForEachAsync(items, async (item, ct) =>
        {
            item.IsScanning = true;
            try
            {
                await Task.Run(() =>
                {
                    if (item.Category == CleanerCategory.DnsCache || item.Category == CleanerCategory.RecycleBin)
                    {
                        item.SizeBytes = 0;
                        return;
                    }

                    string localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
                    string appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);

                    if (item.Category == CleanerCategory.DirectXShader)
                    {
                        long total = CalculateDirectorySizeSafe(Path.Combine(localAppData, "D3DSCache"))
                                   + CalculateDirectorySizeSafe(Path.Combine(localAppData, "Microsoft", "DirectX Shader Cache"));
                        item.SizeBytes = total;
                        return;
                    }

                    if (item.Category == CleanerCategory.NvidiaShader)
                    {
                        long total = CalculateDirectorySizeSafe(Path.Combine(localAppData, "NVIDIA", "DXCache"))
                                   + CalculateDirectorySizeSafe(Path.Combine(localAppData, "NVIDIA", "GLCache"))
                                   + CalculateDirectorySizeSafe(Path.Combine(appData, "NVIDIA", "ComputeCache"));
                        item.SizeBytes = total;
                        return;
                    }

                    if (item.Category == CleanerCategory.AmdShader)
                    {
                        long total = CalculateDirectorySizeSafe(Path.Combine(localAppData, "AMD", "DxCache"))
                                   + CalculateDirectorySizeSafe(Path.Combine(localAppData, "AMD", "GLCache"));
                        item.SizeBytes = total;
                        return;
                    }

                    if (Directory.Exists(item.TargetPath))
                    {
                        item.SizeBytes = CalculateDirectorySizeSafe(item.TargetPath);
                    }
                    else
                    {
                        item.SizeBytes = 0;
                    }
                }, ct);
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
                            try
                            {
                                await _executor.ExecuteCommandAsync("net stop wuauserv");
                                var sdResult = CleanDirectoryContentsSafe(item.TargetPath);
                                cleanedBytes += sdResult.Bytes;
                                filesDeleted += sdResult.Count;
                                errorsCount += sdResult.Errors;
                            }
                            finally
                            {
                                // Ensure Windows Update service is always restarted cleanly
                                await _executor.ExecuteCommandAsync("net start wuauserv");
                            }
                            item.IsCleaned = true;
                            break;

                        case CleanerCategory.DirectXShader:
                            string localAppDataDx = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
                            var dx1 = CleanDirectoryContentsSafe(Path.Combine(localAppDataDx, "D3DSCache"));
                            var dx2 = CleanDirectoryContentsSafe(Path.Combine(localAppDataDx, "Microsoft", "DirectX Shader Cache"));
                            cleanedBytes += dx1.Bytes + dx2.Bytes;
                            filesDeleted += dx1.Count + dx2.Count;
                            errorsCount += dx1.Errors + dx2.Errors;
                            item.IsCleaned = true;
                            break;

                        case CleanerCategory.NvidiaShader:
                            string localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
                            string appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
                            var nv1 = CleanDirectoryContentsSafe(Path.Combine(localAppData, "NVIDIA", "DXCache"));
                            var nv2 = CleanDirectoryContentsSafe(Path.Combine(localAppData, "NVIDIA", "GLCache"));
                            var nv3 = CleanDirectoryContentsSafe(Path.Combine(appData, "NVIDIA", "ComputeCache"));
                            cleanedBytes += nv1.Bytes + nv2.Bytes + nv3.Bytes;
                            filesDeleted += nv1.Count + nv2.Count + nv3.Count;
                            errorsCount += nv1.Errors + nv2.Errors + nv3.Errors;
                            item.IsCleaned = true;
                            break;

                        case CleanerCategory.AmdShader:
                            string amdLocal = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
                            var amd1 = CleanDirectoryContentsSafe(Path.Combine(amdLocal, "AMD", "DxCache"));
                            var amd2 = CleanDirectoryContentsSafe(Path.Combine(amdLocal, "AMD", "GLCache"));
                            cleanedBytes += amd1.Bytes + amd2.Bytes;
                            filesDeleted += amd1.Count + amd2.Count;
                            errorsCount += amd1.Errors + amd2.Errors;
                            item.IsCleaned = true;
                            break;

                        default:
                            if (Directory.Exists(item.TargetPath))
                            {
                                var dirResult = CleanDirectoryContentsSafe(item.TargetPath);
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

    private (long Bytes, int Count, int Errors) CleanDirectoryContentsSafe(string path)
    {
        if (!Directory.Exists(path)) return (0, 0, 0);

        long bytes = 0;
        int count = 0;
        int errors = 0;

        try
        {
            var dirInfo = new DirectoryInfo(path);
            DeleteContentsRecursive(dirInfo, ref bytes, ref count, ref errors);
        }
        catch (Exception ex)
        {
            LogMessage?.Invoke($"[Cleaner Notice] Skipping directory {path}: {ex.Message}");
        }

        return (bytes, count, errors);
    }

    private void DeleteContentsRecursive(DirectoryInfo dir, ref long bytes, ref int count, ref int errors)
    {
        // Safely enumerate files in current level
        FileInfo[]? files = null;
        try
        {
            files = dir.GetFiles();
        }
        catch
        {
            errors++;
            return;
        }

        if (files != null)
        {
            foreach (var file in files)
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
                    // Locked or permission denied in-use file
                    errors++;
                }
            }
        }

        // Safely enumerate and recurse into subdirectories
        DirectoryInfo[]? subDirs = null;
        try
        {
            subDirs = dir.GetDirectories();
        }
        catch
        {
            errors++;
            return;
        }

        if (subDirs != null)
        {
            foreach (var sub in subDirs)
            {
                DeleteContentsRecursive(sub, ref bytes, ref count, ref errors);
                try
                {
                    sub.Delete(true);
                }
                catch
                {
                    // Subdirectory not empty or locked
                }
            }
        }
    }

    private long CalculateDirectorySizeSafe(string path)
    {
        if (!Directory.Exists(path)) return 0;

        long size = 0;
        try
        {
            var root = new DirectoryInfo(path);
            var queue = new Queue<DirectoryInfo>();
            queue.Enqueue(root);

            while (queue.Count > 0)
            {
                var current = queue.Dequeue();

                try
                {
                    foreach (var file in current.EnumerateFiles())
                    {
                        try { size += file.Length; } catch { }
                    }

                    foreach (var dir in current.EnumerateDirectories())
                    {
                        queue.Enqueue(dir);
                    }
                }
                catch
                {
                    // Inaccessible subdirectory ignored safely
                }
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