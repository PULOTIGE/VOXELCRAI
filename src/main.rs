mod archguard;
mod camera;
mod consciousness;
mod engine;
mod evolution;
mod lighting;
mod renderer;
mod simulation;
mod voxel;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    pollster::block_on(engine::run())
}
