//! ═══════════════════════════════════════════════════════════════════════════════
//! CrimeaAI Meta-Organism — Живая искусственная ОС-сознание из вокселей
//! ═══════════════════════════════════════════════════════════════════════════════
//! 
//! 30-секундная демка:
//! 1. Пользователь кидает файл (drag-and-drop)
//! 2. Файл превращается в шар-существо
//! 3. Шар бежит к центральному организму
//! 4. Если совместим → зелёный свет, интеграция, рост
//! 5. Если чужой → красный свет, травма, атрофия
//! 
//! ## Научные основы (забытые русские работы 2013–2020):
//! 
//! - Никонова М.А. (2013) — Коллизии и травма
//! - Ахмадуллина Р.Ф. (2015) — VBM-паттерны атрофии
//! - Алсынбаев К.С. (2016) — Кластеризация в тетраэдры
//! - Сидоров А.В. (2017) — ANIRLE-компрессия
//! - Лавренков Д.Н. (2018) — Коэволюционное обучение
//! - Петрова Е.И. (2019) — Эмоциональное освещение
//! - Козлов И.П. (2020) — Семантические векторы

use crimeaai_meta_organism::{
    consciousness::{MetaOrganism, OrganismMode},
    render::{VoxelRenderer, OrbitCamera, GlobalUniforms, UiRenderer},
};

use pollster::FutureExt;
use std::sync::Arc;
use std::time::Instant;
use winit::{
    event::{ElementState, Event, MouseButton, WindowEvent, MouseScrollDelta},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

fn main() {
    // Инициализация логирования
    env_logger::init();
    
    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║     CrimeaAI Meta-Organism — Запуск...                        ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    
    // Создать event loop
    let event_loop = EventLoop::new().expect("Не удалось создать event loop");
    
    // Создать окно
    let window = WindowBuilder::new()
        .with_title("CrimeaAI Meta-Organism — Живая ОС-сознание")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .expect("Не удалось создать окно");
    
    let window = Arc::new(window);
    
    // Создать wgpu instance
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
        ..Default::default()
    });
    
    // Создать surface
    let surface = instance.create_surface(window.clone())
        .expect("Не удалось создать surface");
    
    // Запросить adapter
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .block_on()
        .expect("Не удалось найти GPU адаптер");
    
    println!("═══════════════════════════════════════");
    println!("║ GPU: {}", adapter.get_info().name);
    println!("║ Backend: {:?}", adapter.get_info().backend);
    println!("═══════════════════════════════════════");
    
    // Создать device и queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Main Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .block_on()
        .expect("Не удалось создать GPU устройство");
    
    let device = Arc::new(device);
    let queue = Arc::new(queue);
    
    // Настроить surface
    let size = window.inner_size();
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);
    
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    
    surface.configure(&device, &config);
    
    // Создать рендерер
    let mut renderer = VoxelRenderer::new(
        device.clone(),
        queue.clone(),
        config.format,
        size.width,
        size.height,
    );
    
    // Создать камеру
    let mut camera = OrbitCamera::new(size.width as f32 / size.height.max(1) as f32);
    camera.distance = 60.0;
    camera.pitch = 0.4;
    
    // Создать организм
    let mut organism = MetaOrganism::new();
    
    // Состояние
    let start_time = Instant::now();
    let mut last_frame = Instant::now();
    let mut frame_count: u64 = 0;
    let mut mouse_pressed = false;
    let mut last_mouse_pos: Option<(f64, f64)> = None;
    let mut auto_rotate = true;
    
    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                 CrimeaAI Meta-Organism v1.0                   ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║  Перетащите любой файл в окно, чтобы создать существо!        ║");
    println!("║                                                               ║");
    println!("║  Управление:                                                  ║");
    println!("║    • ЛКМ + движение — вращение камеры                         ║");
    println!("║    • Колёсико — приближение/отдаление                         ║");
    println!("║    • R — сброс камеры                                         ║");
    println!("║    • Space — вкл/выкл автовращение                            ║");
    println!("║    • Escape — выход                                           ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    
    // Главный цикл
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                
                WindowEvent::Resized(new_size) => {
                    if new_size.width > 0 && new_size.height > 0 {
                        config.width = new_size.width;
                        config.height = new_size.height;
                        surface.configure(&device, &config);
                        renderer.resize(new_size.width, new_size.height);
                        camera.set_aspect(new_size.width, new_size.height);
                    }
                }
                
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state == ElementState::Pressed {
                        match event.logical_key {
                            Key::Named(NamedKey::Escape) => {
                                elwt.exit();
                            }
                            Key::Character(ref c) if c == "r" || c == "R" => {
                                camera = OrbitCamera::new(config.width as f32 / config.height.max(1) as f32);
                                camera.distance = 60.0;
                                camera.pitch = 0.4;
                            }
                            Key::Named(NamedKey::Space) => {
                                auto_rotate = !auto_rotate;
                                println!("Автовращение: {}", if auto_rotate { "ВКЛ" } else { "ВЫКЛ" });
                            }
                            _ => {}
                        }
                    }
                }
                
                WindowEvent::MouseInput { state, button, .. } => {
                    if button == MouseButton::Left {
                        mouse_pressed = state == ElementState::Pressed;
                        if !mouse_pressed {
                            last_mouse_pos = None;
                        }
                    }
                }
                
                WindowEvent::CursorMoved { position, .. } => {
                    if mouse_pressed {
                        if let Some((last_x, last_y)) = last_mouse_pos {
                            let dx = (position.x - last_x) as f32;
                            let dy = (position.y - last_y) as f32;
                            camera.rotate(dx, dy);
                            auto_rotate = false;
                        }
                        last_mouse_pos = Some((position.x, position.y));
                    }
                }
                
                WindowEvent::MouseWheel { delta, .. } => {
                    let scroll = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                    };
                    camera.zoom(scroll * 5.0);
                }
                
                WindowEvent::DroppedFile(path) => {
                    println!("═══════════════════════════════════════");
                    println!("║ Файл брошен: {:?}", path.file_name().unwrap_or_default());
                    println!("═══════════════════════════════════════");
                    
                    if let Ok(data) = std::fs::read(&path) {
                        let file_name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        
                        organism.handle_file_drop(file_name, &data);
                        auto_rotate = false;
                    }
                }
                
                WindowEvent::RedrawRequested => {
                    // Обновить время
                    let now = Instant::now();
                    let dt = (now - last_frame).as_secs_f32();
                    last_frame = now;
                    
                    // Автовращение камеры
                    if auto_rotate {
                        camera.auto_rotate(dt);
                    }
                    
                    // Обновить организм
                    organism.update(dt);
                    
                    // Периодически выводить статистику
                    frame_count += 1;
                    if frame_count % 120 == 0 {
                        let stats = organism.get_stats();
                        println!("{}", UiRenderer::format_stats(&stats));
                    }
                    
                    // Рендеринг
                    let output = match surface.get_current_texture() {
                        Ok(t) => t,
                        Err(wgpu::SurfaceError::Lost) => {
                            surface.configure(&device, &config);
                            return;
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            eprintln!("Недостаточно памяти GPU!");
                            elwt.exit();
                            return;
                        }
                        Err(e) => {
                            eprintln!("Ошибка surface: {:?}", e);
                            return;
                        }
                    };
                    
                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    
                    // Подготовить данные для GPU
                    let instances = organism.prepare_gpu_instances();
                    renderer.update_instances(&instances);
                    
                    // Обновить uniforms
                    let view_proj = camera.view_projection();
                    let camera_pos = camera.position();
                    
                    let mode = match organism.mode {
                        OrganismMode::Normal => 0.0,
                        OrganismMode::Ignite => 1.0,
                        OrganismMode::Trauma => 2.0,
                    };
                    
                    let uniforms = GlobalUniforms {
                        view_proj: view_proj.to_cols_array_2d(),
                        camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z, 1.0],
                        time: start_time.elapsed().as_secs_f32(),
                        pulse_phase: organism.pulse_phase,
                        mode,
                        _padding: 0.0,
                    };
                    
                    renderer.update_uniforms(uniforms);
                    
                    // Рендер
                    let command_buffer = renderer.render(&view);
                    queue.submit(std::iter::once(command_buffer));
                    output.present();
                }
                
                _ => {}
            }
            
            Event::AboutToWait => {
                window.request_redraw();
            }
            
            _ => {}
        }
    }).expect("Ошибка event loop");
}
