// Утилита для baking паттернов освещения
// Запуск: cargo run --bin bake-patterns

use adaptive_entity_engine::pattern_baker::PatternBaker;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lighting Pattern Baker ===");
    println!("Generating baked lighting patterns...");
    println!();
    
    let output_dir = "patterns";
    let baker = PatternBaker::new(output_dir);
    
    baker.bake_all()?;
    
    println!();
    println!("Patterns saved to: {}", output_dir);
    println!("You can now use these patterns in your engine!");
    
    Ok(())
}
