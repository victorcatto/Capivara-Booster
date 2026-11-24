use winreg::enums::*;
use winreg::RegKey;

pub struct RegistryService;

impl RegistryService {
    pub fn read_string(hive: HKEY, path: &str, value: &str) -> Option<String> {
        let key = RegKey::predef(hive).open_subkey(path).ok()?;
        key.get_value(value).ok()
    }

    pub fn read_u32(hive: HKEY, path: &str, value: &str) -> Option<u32> {
        let key = RegKey::predef(hive).open_subkey(path).ok()?;
        key.get_value(value).ok()
    }

    pub fn write_string(hive: HKEY, path: &str, value: &str, data: &str) -> Result<(), std::io::Error> {
        let key = RegKey::predef(hive).open_subkey_with_flags(path, KEY_SET_VALUE)?;
        key.set_value(value, &data)
    }

    pub fn write_u32(hive: HKEY, path: &str, value: &str, data: u32) -> Result<(), std::io::Error> {
        let key = RegKey::predef(hive).open_subkey_with_flags(path, KEY_SET_VALUE)?;
        key.set_value(value, &data)
    }
}
