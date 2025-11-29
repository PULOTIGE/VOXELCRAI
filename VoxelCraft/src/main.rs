// VoxelCraft - Minecraft-like game
// Desktop entry point

#[cfg(not(target_os = "android"))]
fn main() {
    env_logger::init();
    voxelcraft::run_game();
}

#[cfg(target_os = "android")]
fn main() {
    // Android uses android_main from lib.rs
}
