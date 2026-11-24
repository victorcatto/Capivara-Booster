using System.IO;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using CapivaraBooster.Models;

namespace CapivaraBooster.Services;

public class AuthService : IAuthService
{
    private const string MasterAdminEmailHash = "7be2176cae040076deaf378728a219a8fd604850e6b521e83e13e324c1d4ecf9";

    private static readonly string[] AllowedDomains =
    {
        "@gmail.com",
        "@outlook.com",
        "@yahoo.com.br",
        "@yahoo.com"
    };

    private readonly string _appDataFolder;
    private readonly string _sessionFilePath;
    private readonly string _localVaultPath;
    private readonly ILocalizationService _loc;
    private readonly object _vaultLock = new();

    private UserAccount? _currentUser;

    public UserAccount? CurrentUser => _currentUser;
    public bool IsAuthenticated => _currentUser != null;
    public bool IsAdmin => _currentUser?.IsAdmin ?? false;

    public event Action<UserAccount?>? AuthStateChanged;

    public AuthService(ILocalizationService loc)
    {
        _loc = loc;
        _appDataFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "CapivaraBooster");
        Directory.CreateDirectory(_appDataFolder);
        _sessionFilePath = Path.Combine(_appDataFolder, "session.dat");
        _localVaultPath = Path.Combine(_appDataFolder, "users_vault.dat");

        TryRestoreSession();
    }

    public bool IsDomainAllowed(string email)
    {
        if (string.IsNullOrWhiteSpace(email)) return false;

        string trimmed = email.Trim().ToLowerInvariant();
        int atIndex = trimmed.IndexOf('@');
        if (atIndex <= 0 || atIndex == trimmed.Length - 1)
        {
            // Must have a valid username before '@' and domain after
            return false;
        }

        return AllowedDomains.Any(domain => trimmed.EndsWith(domain, StringComparison.OrdinalIgnoreCase));
    }

    public async Task<(bool Success, string Message)> RegisterAsync(string email, string password)
    {
        return await Task.Run(() =>
        {
            if (string.IsNullOrWhiteSpace(email) || string.IsNullOrWhiteSpace(password))
            {
                return (false, _loc.IsPortuguese
                    ? "E-mail e senha são obrigatórios."
                    : "Email and password are required.");
            }

            string normalizedEmail = email.Trim().ToLowerInvariant();

            // 1. Strict Domain Validation
            if (!IsDomainAllowed(normalizedEmail))
            {
                return (false, _loc.IsPortuguese
                    ? "Domínio de e-mail não autorizado. O registro é permitido exclusivamente para: @gmail.com, @outlook.com, @yahoo.com.br e @yahoo.com."
                    : "Unauthorized email domain. Registration is exclusively permitted for: @gmail.com, @outlook.com, @yahoo.com.br, and @yahoo.com.");
            }

            if (password.Length < 6)
            {
                return (false, _loc.IsPortuguese
                    ? "A senha deve conter no mínimo 6 caracteres."
                    : "Password must contain at least 6 characters.");
            }

            lock (_vaultLock)
            {
                // 2. Load Local Vault
                var vault = LoadVault();
                if (vault.ContainsKey(normalizedEmail))
                {
                    return (false, _loc.IsPortuguese
                        ? "Este e-mail já está registrado."
                        : "This email address is already registered.");
                }

                // 3. Hash Password with PBKDF2 (100k iterations)
                byte[] salt = RandomNumberGenerator.GetBytes(16);
                byte[] hash = Rfc2898DeriveBytes.Pbkdf2(
                    Encoding.UTF8.GetBytes(password),
                    salt,
                    iterations: 100_000,
                    HashAlgorithmName.SHA256,
                    outputLength: 32);

                string saltHex = Convert.ToHexString(salt);
                string hashHex = Convert.ToHexString(hash);

                // Role assignment uses a private deterministic administrator identity hash
                var role = IsMasterAdmin(normalizedEmail)
                    ? UserRole.Admin
                    : UserRole.StandardUser;

                vault[normalizedEmail] = $"{saltHex}:{hashHex}:{(int)role}";
                SaveVault(vault);

                // 5. Automatically log in after registration
                SetCurrentUser(normalizedEmail, role, rememberMe: true);

                return (true, _loc.IsPortuguese
                    ? "Conta registrada com sucesso!"
                    : "Account registered successfully!");
            }
        });
    }

    public async Task<(bool Success, string Message)> LoginAsync(string email, string password, bool rememberMe = true)
    {
        return await Task.Run(() =>
        {
            if (string.IsNullOrWhiteSpace(email) || string.IsNullOrWhiteSpace(password))
            {
                return (false, _loc.IsPortuguese
                    ? "Informe o e-mail e a senha."
                    : "Please enter your email and password.");
            }

            string normalizedEmail = email.Trim().ToLowerInvariant();

            // Domain validation check
            if (!IsDomainAllowed(normalizedEmail))
            {
                return (false, _loc.IsPortuguese
                    ? "Domínio não permitido. Acesso restrito a: @gmail.com, @outlook.com, @yahoo.com.br e @yahoo.com."
                    : "Domain not allowed. Access restricted to: @gmail.com, @outlook.com, @yahoo.com.br, and @yahoo.com.");
            }

            lock (_vaultLock)
            {
                var vault = LoadVault();

                // Master admin bootstrapping if vault does not contain master record yet
                if (IsMasterAdmin(normalizedEmail) && !vault.ContainsKey(normalizedEmail))
                {
                    byte[] salt = RandomNumberGenerator.GetBytes(16);
                    byte[] hash = Rfc2898DeriveBytes.Pbkdf2(
                        Encoding.UTF8.GetBytes(password),
                        salt,
                        100_000,
                        HashAlgorithmName.SHA256,
                        32);
                    vault[normalizedEmail] = $"{Convert.ToHexString(salt)}:{Convert.ToHexString(hash)}:{(int)UserRole.Admin}";
                    SaveVault(vault);
                }

                if (!vault.TryGetValue(normalizedEmail, out string? entry) || string.IsNullOrEmpty(entry))
                {
                    return (false, _loc.IsPortuguese
                        ? "Usuário não encontrado ou senha incorreta."
                        : "User not found or incorrect password.");
                }

                var parts = entry.Split(':');
                if (parts.Length < 3)
                {
                    return (false, "Corrupted credential record.");
                }

                byte[] storedSalt = Convert.FromHexString(parts[0]);
                byte[] storedHash = Convert.FromHexString(parts[1]);
                int roleVal = int.Parse(parts[2]);

                byte[] computedHash = Rfc2898DeriveBytes.Pbkdf2(
                    Encoding.UTF8.GetBytes(password),
                    storedSalt,
                    100_000,
                    HashAlgorithmName.SHA256,
                    32);

                if (!CryptographicOperations.FixedTimeEquals(storedHash, computedHash))
                {
                    return (false, _loc.IsPortuguese
                        ? "Senha incorreta."
                        : "Incorrect password.");
                }

                // Verify the private single-administrator rule
                UserRole assignedRole = (roleVal == (int)UserRole.Admin && IsMasterAdmin(normalizedEmail))
                    ? UserRole.Admin
                    : UserRole.StandardUser;

                SetCurrentUser(normalizedEmail, assignedRole, rememberMe);

                return (true, _loc.IsPortuguese
                    ? "Login efetuado com sucesso!"
                    : "Login successful!");
            }
        });
    }

    public void Logout()
    {
        _currentUser = null;
        try
        {
            if (File.Exists(_sessionFilePath))
            {
                File.Delete(_sessionFilePath);
            }
        }
        catch { }

        AuthStateChanged?.Invoke(null);
    }

    private void SetCurrentUser(string email, UserRole role, bool rememberMe)
    {
        _currentUser = new UserAccount
        {
            Email = email,
            DisplayName = email.Split('@')[0],
            Role = role,
            Token = Guid.NewGuid().ToString("N"),
            LastLogin = DateTime.Now
        };

        if (rememberMe)
        {
            SaveSession(_currentUser);
        }

        AuthStateChanged?.Invoke(_currentUser);
    }

    private void SaveSession(UserAccount user)
    {
        try
        {
            string json = JsonSerializer.Serialize(user);
            byte[] plainBytes = Encoding.UTF8.GetBytes(json);
            byte[] encrypted = ProtectedData.Protect(plainBytes, null, DataProtectionScope.CurrentUser);
            File.WriteAllBytes(_sessionFilePath, encrypted);
        }
        catch { }
    }

    private void TryRestoreSession()
    {
        try
        {
            if (File.Exists(_sessionFilePath))
            {
                byte[] encrypted = File.ReadAllBytes(_sessionFilePath);
                byte[] decrypted = ProtectedData.Unprotect(encrypted, null, DataProtectionScope.CurrentUser);
                string json = Encoding.UTF8.GetString(decrypted);
                var user = JsonSerializer.Deserialize<UserAccount>(json);
                if (user != null && IsDomainAllowed(user.Email))
                {
                    // Strictly enforce single admin rule on session restore
                    if (user.Role == UserRole.Admin && !IsMasterAdmin(user.Email))
                    {
                        user.Role = UserRole.StandardUser;
                    }

                    _currentUser = user;
                }
            }
        }
        catch { }
    }

    private Dictionary<string, string> LoadVault()
    {
        try
        {
            if (File.Exists(_localVaultPath))
            {
                byte[] encrypted = File.ReadAllBytes(_localVaultPath);
                byte[] decrypted = ProtectedData.Unprotect(encrypted, null, DataProtectionScope.CurrentUser);
                string json = Encoding.UTF8.GetString(decrypted);
                return JsonSerializer.Deserialize<Dictionary<string, string>>(json) ?? new();
            }
        }
        catch { }

        return new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
    }

    private static bool IsMasterAdmin(string email)
    {
        using var sha = SHA256.Create();
        string hash = Convert.ToHexString(sha.ComputeHash(Encoding.UTF8.GetBytes(email.Trim().ToLowerInvariant()))).ToLowerInvariant();
        return string.Equals(hash, MasterAdminEmailHash, StringComparison.OrdinalIgnoreCase);
    }

    private void SaveVault(Dictionary<string, string> vault)
    {
        try
        {
            string json = JsonSerializer.Serialize(vault);
            byte[] plain = Encoding.UTF8.GetBytes(json);
            byte[] encrypted = ProtectedData.Protect(plain, null, DataProtectionScope.CurrentUser);
            File.WriteAllBytes(_localVaultPath, encrypted);
        }
        catch { }
    }
}