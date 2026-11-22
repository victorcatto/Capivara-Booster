namespace CapivaraBooster.Common;

public enum LogLevel
{
    Info,
    Success,
    Warning,
    Error,
    Command
}

public class LogItem
{
    public DateTime Timestamp { get; } = DateTime.Now;
    public LogLevel Level { get; init; }
    public string Message { get; init; } = string.Empty;

    public string TimeString => Timestamp.ToString("HH:mm:ss");

    public string Prefix => Level switch
    {
        LogLevel.Info => "[INFO]",
        LogLevel.Success => "[OK]",
        LogLevel.Warning => "[WARN]",
        LogLevel.Error => "[ERROR]",
        LogLevel.Command => "[CMD]",
        _ => "[LOG]"
    };

    public string FormattedMessage => $"{TimeString} {Prefix} {Message}";

    public string ColorHex => Level switch
    {
        LogLevel.Info => "#8E92A4",      // Muted cool gray
        LogLevel.Success => "#00E676",   // Neon green
        LogLevel.Warning => "#FFB300",   // Amber
        LogLevel.Error => "#FF1744",     // Neon coral red
        LogLevel.Command => "#00E5FF",   // Cyber cyan neon
        _ => "#F4F4F6"
    };
}
