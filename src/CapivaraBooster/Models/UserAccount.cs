namespace CapivaraBooster.Models;

public enum UserRole
{
    StandardUser,
    Admin
}

public class UserAccount
{
    public string Id { get; set; } = Guid.NewGuid().ToString("N");
    public string Email { get; set; } = string.Empty;
    public string DisplayName { get; set; } = string.Empty;
    public UserRole Role { get; set; } = UserRole.StandardUser;
    public bool IsAdmin => Role == UserRole.Admin;
    public string Token { get; set; } = string.Empty;
    public DateTime LastLogin { get; set; } = DateTime.Now;
}
