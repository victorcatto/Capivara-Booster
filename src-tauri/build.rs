fn main() {
    // requireAdministrator no manifesto: sem elevação real cada escrita em HKLM
    // falha com "Acesso negado (os error 5)" (GameDVR, IFEO, bcdedit).
    let windows = tauri_build::WindowsAttributes::new().app_manifest(include_str!("app.manifest"));
    let attrs = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attrs).expect("falha ao compilar o projeto Tauri");
}
