using System.Diagnostics;
using System.Text;

namespace CapivaraBooster.Services;

public class ProcessExecutor : IProcessExecutor
{
    public event Action<string, bool>? OutputReceived;

    public async Task<ExecutionResult> ExecuteCommandAsync(string command, CancellationToken ct = default)
    {
        OutputReceived?.Invoke($"cmd /c {command}", false);
        return await RunProcessInternalAsync("cmd.exe", $"/c \"{command}\"", ct);
    }

    public async Task<ExecutionResult> ExecutePowerShellAsync(string script, CancellationToken ct = default)
    {
        OutputReceived?.Invoke($"powershell.exe -Command \"{script}\"", false);
        string arguments = $"-NoProfile -NonInteractive -ExecutionPolicy Bypass -Command \"{script}\"";
        return await RunProcessInternalAsync("powershell.exe", arguments, ct);
    }

    private async Task<ExecutionResult> RunProcessInternalAsync(string fileName, string arguments, CancellationToken ct)
    {
        var stdoutBuilder = new StringBuilder();
        var stderrBuilder = new StringBuilder();

        var psi = new ProcessStartInfo
        {
            FileName = fileName,
            Arguments = arguments,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true,
            StandardOutputEncoding = Encoding.UTF8,
            StandardErrorEncoding = Encoding.UTF8
        };

        using var process = new Process { StartInfo = psi };

        process.OutputDataReceived += (_, e) =>
        {
            if (e.Data != null)
            {
                stdoutBuilder.AppendLine(e.Data);
                OutputReceived?.Invoke(e.Data, false);
            }
        };

        process.ErrorDataReceived += (_, e) =>
        {
            if (e.Data != null)
            {
                stderrBuilder.AppendLine(e.Data);
                OutputReceived?.Invoke(e.Data, true);
            }
        };

        try
        {
            process.Start();
            process.BeginOutputReadLine();
            process.BeginErrorReadLine();

            await process.WaitForExitAsync(ct);
            return new ExecutionResult(process.ExitCode, stdoutBuilder.ToString().Trim(), stderrBuilder.ToString().Trim());
        }
        catch (OperationCanceledException)
        {
            try { process.Kill(true); } catch { }
            return new ExecutionResult(-1, stdoutBuilder.ToString(), "Operation timed out or was canceled.");
        }
        catch (Exception ex)
        {
            string err = $"Process execution failure: {ex.Message}";
            OutputReceived?.Invoke(err, true);
            return new ExecutionResult(-1, stdoutBuilder.ToString(), err);
        }
    }
}
