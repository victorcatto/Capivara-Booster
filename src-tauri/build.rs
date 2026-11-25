fn main() {
    // Embute app.manifest (requireAdministrator): sem isso o exe sobe sem elevação
    // e cada escrita em HKLM falha com "Acesso negado (os error 5)".
    static MANIFEST: &str = include_str!("app.manifest");
    embed_resource::compile(MANIFEST, embed_resource::NONE).unwrap();
    tauri_build::build()
}
