using CapivaraBooster.Models;

namespace CapivaraBooster.Services;

public interface IAuthService
{
    UserAccount? CurrentUser { get; }
    bool IsAuthenticated { get; }
    bool IsAdmin { get; }

    event Action<UserAccount?>? AuthStateChanged;

    Task<(bool Success, string Message)> RegisterAsync(string email, string password);
    Task<(bool Success, string Message)> LoginAsync(string email, string password, bool rememberMe = true);
    void Logout();
    bool IsDomainAllowed(string email);
}
