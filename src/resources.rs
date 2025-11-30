// Встроенные ресурсы в EXE файл

#[cfg(feature = "gui")]
use include_dir::{include_dir, Dir};

#[cfg(feature = "gui")]
// Встраиваем шейдеры
pub const SHADERS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/shaders");

// Получить содержимое шейдера
#[cfg(feature = "gui")]
pub fn get_shader(name: &str) -> Option<&'static str> {
    SHADERS_DIR
        .get_file(name)
        .and_then(|f| f.contents_utf8())
}

// Получить все шейдеры
#[cfg(feature = "gui")]
pub fn list_shaders() -> Vec<&'static str> {
    SHADERS_DIR
        .files()
        .filter_map(|f| f.path().file_name())
        .filter_map(|n| n.to_str())
        .collect()
}

#[cfg(not(feature = "gui"))]
pub fn get_shader(_name: &str) -> Option<&'static str> {
    None
}

#[cfg(not(feature = "gui"))]
pub fn list_shaders() -> Vec<&'static str> {
    Vec::new()
}
