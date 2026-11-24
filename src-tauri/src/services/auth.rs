use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::services::registry::dirs_roaming_data;

const MASTER_ADMIN_EMAIL: &str = "victorcatto93@gmail.com";
const ALLOWED_DOMAINS: [&str; 4] = ["@gmail.com", "@outlook.com", "@yahoo.com.br", "@yahoo.com"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
 pub email: String,
 pub display_name: String,
 pub role: String,
 pub token: String,
 pub last_login: String,
}

pub struct AuthService {
 vault_path: PathBuf,
 session_path: PathBuf,
 current: Option<UserAccount>,
}

impl AuthService {
 pub fn new() -> Self {
 let folder = dirs_roaming_data().join("CapivaraBooster");
 let _ = std::fs::create_dir_all(&folder);
 let vault_path = folder.join("users_vault.dat");
 let session_path = folder.join("session.dat");
 let mut svc = Self { vault_path, session_path, current: None };
 svc.try_restore_session();
 svc
 }

 pub fn current(&self) -> Option<&UserAccount> { self.current.as_ref() }
 pub fn is_authenticated(&self) -> bool { self.current.is_some() }
 pub fn is_admin(&self) -> bool { self.current.as_ref().map(|u| u.role == "Admin").unwrap_or(false) }

 pub fn is_domain_allowed(email: &str) -> bool {
 let e = email.trim().to_lowercase();
 if let Some(at) = e.find('@') {
 if at == 0 || at == e.len() - 1 { return false; }
 return ALLOWED_DOMAINS.iter().any(|d| e.ends_with(d));
 }
 false
 }

 pub fn register(&mut self, email: &str, password: &str) -> Result<UserAccount, String> {
 let normalized = email.trim().to_lowercase();
 if normalized.is_empty() || password.is_empty() { return Err("e-mail e senha obrigatórios".into()); }
 if !Self::is_domain_allowed(&normalized) { return Err("domínio de e-mail não autorizado".into()); }
 if password.len() < 6 { return Err("senha deve ter no mínimo 6 caracteres".into()); }

 let mut vault = self.load_vault();
 if vault.contains_key(&normalized) { return Err("e-mail já registrado".into()); }

 let salt = format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
 let hash = Self::pbkdf2_simple(password, &salt);
 let role = if normalized.eq_ignore_ascii_case(MASTER_ADMIN_EMAIL) { "Admin" } else { "StandardUser" };
 vault.insert(normalized.clone(), format!("{salt}:{hash}:{role}"));
 self.save_vault(&vault);

 let user = self.login(&normalized, password)?;
 Ok(user)
 }

 pub fn login(&mut self, email: &str, password: &str) -> Result<UserAccount, String> {
 let normalized = email.trim().to_lowercase();
 if normalized.is_empty() || password.is_empty() { return Err("informe e-mail e senha".into()); }
 if !Self::is_domain_allowed(&normalized) { return Err("domínio não permitido".into()); }

 let mut vault = self.load_vault();
 let entry = match vault.get(&normalized) { Some(e) => e.clone(), None => return Err("usuário não encontrado".into()) };
 let parts: Vec<&str> = entry.split(':').collect();
 if parts.len() < 3 { return Err("registro corrompido".into()); }
 let (salt, hash, role_str) = (parts[0], parts[1], parts[2]);

 let computed = Self::pbkdf2_simple(password, salt);
 if computed != hash { return Err("senha incorreta".into()); }

 let role = if normalized.eq_ignore_ascii_case(MASTER_ADMIN_EMAIL) { "Admin".to_string() } else { role_str.to_string() };
 let user = UserAccount {
 email: normalized.clone(),
 display_name: normalized.split('@').next().unwrap_or("user").to_string(),
 role,
 token: uuid_simple(),
 last_login: chrono::Utc::now().to_rfc3339(),
 };

 self.current = Some(user.clone());
 self.save_session(&user);
 Ok(user)
 }

 pub fn logout(&mut self) {
 self.current = None;
 let _ = std::fs::remove_file(&self.session_path);
 }

 fn try_restore_session(&mut self) {
 if let Ok(json) = std::fs::read_to_string(&self.session_path) {
 if let Ok(user) = serde_json::from_str::<UserAccount>(&json) {
 if Self::is_domain_allowed(&user.email) {
 let mut u = user;
 if u.role == "Admin" && !u.email.eq_ignore_ascii_case(MASTER_ADMIN_EMAIL) { u.role = "StandardUser".into(); }
 self.current = Some(u);
 }
 }
 }
 }

 fn save_session(&self, user: &UserAccount) {
 if let Ok(json) = serde_json::to_string(user) {
 let _ = std::fs::write(&self.session_path, json);
 }
 }

 fn load_vault(&self) -> std::collections::HashMap<String, String> {
 std::fs::read_to_string(&self.vault_path)
 .ok()
 .and_then(|j| serde_json::from_str(&j).ok())
 .unwrap_or_default()
 }

 fn save_vault(&self, vault: &std::collections::HashMap<String, String>) {
 if let Ok(json) = serde_json::to_string_pretty(vault) {
 let _ = std::fs::write(&self.vault_path, json);
 }
 }

 fn pbkdf2_simple(password: &str, salt: &str) -> String {
 let mut hasher = Sha256::new();
 for _ in 0..100_000 {
 hasher.update(format!("{password}:{salt}:capivara"));
 }
 format!("{:x}", hasher.finalize())
 }
}

fn uuid_simple() -> String {
 use sha2::{Digest, Sha256};
 let mut h = Sha256::new();
 h.update(format!("{:?}", std::time::SystemTime::now()));
 format!("{:x}", h.finalize())[..32].to_string()
}