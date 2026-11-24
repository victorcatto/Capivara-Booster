use std::collections::HashMap;
use std::path::PathBuf;
use base64::Engine;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
};
use crate::services::registry::dirs_roaming_data;

const MASTER_ADMIN_EMAIL: &str = "victorcatto93@gmail.com";
const ALLOWED_DOMAINS: [&str; 4] = ["@gmail.com", "@outlook.com", "@yahoo.com.br", "@yahoo.com"];
const PBKDF2_ITERATIONS: u32 = 100_000;

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
        let mut service = Self { vault_path, session_path, current: None };
        service.try_restore_session();
        service
    }

    pub fn current(&self) -> Option<&UserAccount> { self.current.as_ref() }
    pub fn is_authenticated(&self) -> bool { self.current.is_some() }
    pub fn is_admin(&self) -> bool { self.current.as_ref().is_some_and(|u| u.role == "Admin") }

    pub fn is_domain_allowed(email: &str) -> bool {
        let email = email.trim();
        let Some((local, domain)) = email.rsplit_once('@') else { return false };
        !local.is_empty() && ALLOWED_DOMAINS.iter().any(|d| domain.eq_ignore_ascii_case(&d[1..]))
    }

    pub fn register(&mut self, email: &str, password: &str) -> Result<UserAccount, String> {
        let normalized = email.trim().to_lowercase();
        if normalized.is_empty() || password.is_empty() { return Err("e-mail e senha são obrigatórios".into()); }
        if !Self::is_domain_allowed(&normalized) { return Err("domínio de e-mail não autorizado".into()); }
        if password.len() < 6 { return Err("a senha deve conter no mínimo 6 caracteres".into()); }

        let mut vault = self.load_vault()?;
        if vault.contains_key(&normalized) { return Err("este e-mail já está registrado".into()); }

        let mut salt = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);
        let mut hash = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut hash);
        let role = if normalized.eq_ignore_ascii_case(MASTER_ADMIN_EMAIL) { "Admin" } else { "StandardUser" };
        vault.insert(normalized.clone(), format!("{}:{}:{}", hex(&salt), hex(&hash), role));
        self.save_vault(&vault)?;
        self.login(&normalized, password)
    }

    pub fn login(&mut self, email: &str, password: &str) -> Result<UserAccount, String> {
        let normalized = email.trim().to_lowercase();
        if normalized.is_empty() || password.is_empty() { return Err("informe o e-mail e a senha".into()); }
        if !Self::is_domain_allowed(&normalized) { return Err("domínio de e-mail não permitido".into()); }

        let vault = self.load_vault()?;
        let entry = vault.get(&normalized).ok_or("usuário não encontrado ou senha incorreta")?;
        let mut parts = entry.splitn(3, ':');
        let salt = decode_hex(parts.next().ok_or("registro de credencial corrompido")?)?;
        let expected = decode_hex(parts.next().ok_or("registro de credencial corrompido")?)?;
        let stored_role = parts.next().ok_or("registro de credencial corrompido")?;
        if salt.len() != 16 || expected.len() != 32 { return Err("registro de credencial corrompido".into()); }

        let mut actual = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut actual);
        if !constant_time_eq(&actual, &expected) { return Err("senha incorreta".into()); }

        let role = if normalized.eq_ignore_ascii_case(MASTER_ADMIN_EMAIL) { "Admin" } else { stored_role };
        let user = UserAccount {
            email: normalized.clone(),
            display_name: normalized.split('@').next().unwrap_or("user").to_string(),
            role: role.to_string(),
            token: uuid::Uuid::new_v4().simple().to_string(),
            last_login: chrono::Utc::now().to_rfc3339(),
        };
        self.current = Some(user.clone());
        self.save_session(&user)?;
        Ok(user)
    }

    pub fn logout(&mut self) {
        self.current = None;
        let _ = std::fs::remove_file(&self.session_path);
    }

    fn try_restore_session(&mut self) {
        let Ok(encrypted) = std::fs::read(&self.session_path) else { return };
        let Ok(plain) = dpapi_unprotect(&encrypted) else { return };
        let Ok(user) = serde_json::from_slice::<UserAccount>(&plain) else { return };
        if !Self::is_domain_allowed(&user.email) { return; }
        let mut user = user;
        if user.role == "Admin" && !user.email.eq_ignore_ascii_case(MASTER_ADMIN_EMAIL) {
            user.role = "StandardUser".into();
        }
        self.current = Some(user);
    }

    fn save_session(&self, user: &UserAccount) -> Result<(), String> {
        let plain = serde_json::to_vec(user).map_err(|e| e.to_string())?;
        let encrypted = dpapi_protect(&plain)?;
        std::fs::write(&self.session_path, encrypted).map_err(|e| format!("não foi possível salvar a sessão: {e}"))
    }

    fn load_vault(&self) -> Result<HashMap<String, String>, String> {
        let encrypted = match std::fs::read(&self.vault_path) {
            Ok(data) => data,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(HashMap::new()),
            Err(e) => return Err(format!("não foi possível ler o cofre de contas: {e}")),
        };
        let plain = dpapi_unprotect(&encrypted)?;
        serde_json::from_slice(&plain).map_err(|e| format!("cofre de contas inválido: {e}"))
    }

    fn save_vault(&self, vault: &HashMap<String, String>) -> Result<(), String> {
        let plain = serde_json::to_vec(vault).map_err(|e| e.to_string())?;
        let encrypted = dpapi_protect(&plain)?;
        std::fs::write(&self.vault_path, encrypted).map_err(|e| format!("não foi possível salvar o cofre de contas: {e}"))
    }
}

fn dpapi_protect(input: &[u8]) -> Result<Vec<u8>, String> {
    unsafe {
        let mut input_blob = CRYPT_INTEGER_BLOB { cbData: input.len() as u32, pbData: input.as_ptr() as *mut u8 };
        let mut output_blob = CRYPT_INTEGER_BLOB::default();
        CryptProtectData(&mut input_blob, None, None, None, None, CRYPTPROTECT_UI_FORBIDDEN, &mut output_blob)
            .map_err(|e| format!("DPAPI ProtectData falhou: {e}"))?;
        let output = std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize).to_vec();
        let _ = windows::Win32::System::Memory::LocalFree(windows::Win32::Foundation::HLOCAL(output_blob.pbData as *mut _));
        Ok(output)
    }
}

fn dpapi_unprotect(input: &[u8]) -> Result<Vec<u8>, String> {
    unsafe {
        let mut input_blob = CRYPT_INTEGER_BLOB { cbData: input.len() as u32, pbData: input.as_ptr() as *mut u8 };
        let mut output_blob = CRYPT_INTEGER_BLOB::default();
        CryptUnprotectData(&mut input_blob, None, None, None, None, CRYPTPROTECT_UI_FORBIDDEN, &mut output_blob)
            .map_err(|e| format!("DPAPI UnprotectData falhou: {e}"))?;
        let output = std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize).to_vec();
        let _ = windows::Win32::System::Memory::LocalFree(windows::Win32::Foundation::HLOCAL(output_blob.pbData as *mut _));
        Ok(output)
    }
}

fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 { return Err("registro de credencial corrompido".into()); }
    (0..value.len()).step_by(2).map(|i| u8::from_str_radix(&value[i..i + 2], 16).map_err(|_| "registro de credencial corrompido".into())).collect()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
