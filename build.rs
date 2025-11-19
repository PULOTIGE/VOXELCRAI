fn main() {
    // Build script for Adaptive Entity Engine v1.0
    println!("cargo:rerun-if-changed=build.rs");
    
    // Embed shaders
    println!("cargo:rerun-if-changed=src/shaders/point_cloud.wgsl");
}
