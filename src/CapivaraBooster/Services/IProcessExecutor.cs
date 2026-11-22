namespace CapivaraBooster.Services;

public record ExecutionResult(int ExitCode, string StandardOutput, string StandardError)
{
    public bool IsSuccess => ExitCode == 0;
}

public interface IProcessExecutor
{
    event Action<string, bool>? OutputReceived; // message, isError

    Task<ExecutionResult> ExecuteCommandAsync(string command, CancellationToken ct = default);
    Task<ExecutionResult> ExecutePowerShellAsync(string script, CancellationToken ct = default);
}
