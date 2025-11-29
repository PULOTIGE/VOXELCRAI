// ═══════════════════════════════════════════════════════════════════════════════
// CrimeaAI Meta-Organism — Evolution Compute Shader
// ═══════════════════════════════════════════════════════════════════════════════
// 
// Параллельное обновление состояния вокселей на GPU.
// 
// Источники:
// - Лавренков Д.Н. (2018) — Коэволюционное обучение
// - Никонова М.А. (2013) — Распространение травмы

// ═══════════════════════════════════════════════════════════════════════════════
// Структуры данных
// ═══════════════════════════════════════════════════════════════════════════════

struct Voxel {
    position: vec3<f32>,
    _pad0: f32,
    
    velocity: vec3<f32>,
    _pad1: f32,
    
    // Семантический вектор (8 x f16 упакован в 4 x u32)
    semantic_01: u32,  // sem[0], sem[1]
    semantic_23: u32,  // sem[2], sem[3]
    semantic_45: u32,  // sem[4], sem[5]
    semantic_67: u32,  // sem[6], sem[7]
    
    energy: f32,
    trauma_level: f32,
    atrophy_factor: f32,
    lifetime: f32,
    
    // Connections (6 x u16 упакован в 3 x u32)
    connections_01: u32,
    connections_23: u32,
    connections_45: u32,
    
    flags: u32,
    cluster_id: u32,
    creature_id: u32,
    _pad2: u32,
}

struct SimParams {
    delta_time: f32,
    time: f32,
    recovery_rate: f32,
    atrophy_rate: f32,
    
    trauma_threshold: f32,
    energy_protection: f32,
    learning_rate: f32,
    num_voxels: u32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Буферы
// ═══════════════════════════════════════════════════════════════════════════════

@group(0) @binding(0)
var<storage, read_write> voxels: array<Voxel>;

@group(0) @binding(1)
var<uniform> params: SimParams;

// ═══════════════════════════════════════════════════════════════════════════════
// Вспомогательные функции
// ═══════════════════════════════════════════════════════════════════════════════

// Распаковать u16 из u32
fn unpack_u16(packed: u32, idx: u32) -> u32 {
    if idx == 0u {
        return packed & 0xFFFFu;
    } else {
        return (packed >> 16u) & 0xFFFFu;
    }
}

// Флаги вокселей
const FLAG_ALIVE: u32 = 1u;
const FLAG_MOVING: u32 = 2u;
const FLAG_IGNITED: u32 = 4u;
const FLAG_TRAUMATIZED: u32 = 8u;
const FLAG_DEAD: u32 = 16u;

// ═══════════════════════════════════════════════════════════════════════════════
// Main Compute Shader
// ═══════════════════════════════════════════════════════════════════════════════

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    
    if idx >= params.num_voxels {
        return;
    }
    
    var voxel = voxels[idx];
    
    // Пропустить мёртвые воксели
    if (voxel.flags & FLAG_ALIVE) == 0u {
        return;
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 1. Физика движения
    // ═══════════════════════════════════════════════════════════════════════════
    
    if (voxel.flags & FLAG_MOVING) != 0u {
        voxel.position += voxel.velocity * params.delta_time;
        
        // Затухание скорости
        voxel.velocity *= 0.99;
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 2. Восстановление от травмы (Никонова 2013)
    // ═══════════════════════════════════════════════════════════════════════════
    
    if voxel.trauma_level > 0.0 {
        voxel.trauma_level = max(voxel.trauma_level - params.recovery_rate * params.delta_time, 0.0);
        
        if voxel.trauma_level < 0.05 {
            voxel.flags &= ~FLAG_TRAUMATIZED;
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 3. Атрофия (Ахмадуллина 2015)
    // ═══════════════════════════════════════════════════════════════════════════
    
    let should_atrophy = voxel.trauma_level > params.trauma_threshold ||
                         voxel.energy < params.energy_protection;
    
    if should_atrophy {
        let rate = params.atrophy_rate * params.delta_time * (1.0 + voxel.trauma_level);
        voxel.atrophy_factor = min(voxel.atrophy_factor + rate, 1.0);
        
        // Смерть при высокой атрофии
        if voxel.atrophy_factor > 0.95 {
            voxel.flags |= FLAG_DEAD;
            voxel.flags &= ~FLAG_ALIVE;
        }
    } else if voxel.energy > 0.7 && voxel.trauma_level < 0.1 {
        // Восстановление
        voxel.atrophy_factor = max(voxel.atrophy_factor - params.atrophy_rate * params.delta_time * 0.3, 0.0);
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 4. Хебианское обучение связей (Лавренков 2018)
    // ═══════════════════════════════════════════════════════════════════════════
    
    // Собрать энергию от соседей
    var neighbor_energy = 0.0;
    var neighbor_count = 0u;
    
    // Connection 0
    let conn0 = unpack_u16(voxel.connections_01, 0u);
    if conn0 < params.num_voxels && conn0 != 0xFFFFu {
        neighbor_energy += voxels[conn0].energy;
        neighbor_count += 1u;
    }
    
    // Connection 1
    let conn1 = unpack_u16(voxel.connections_01, 1u);
    if conn1 < params.num_voxels && conn1 != 0xFFFFu {
        neighbor_energy += voxels[conn1].energy;
        neighbor_count += 1u;
    }
    
    // Усилить энергию если соседи тоже активны
    if neighbor_count > 0u {
        let avg_neighbor = neighbor_energy / f32(neighbor_count);
        let correlation = voxel.energy * avg_neighbor;
        
        if correlation > 0.5 {
            voxel.energy = min(voxel.energy + params.learning_rate * params.delta_time * 0.1, 2.0);
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 5. Обновить время жизни
    // ═══════════════════════════════════════════════════════════════════════════
    
    voxel.lifetime += params.delta_time;
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Записать результат
    // ═══════════════════════════════════════════════════════════════════════════
    
    voxels[idx] = voxel;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Дополнительный шейдер для распространения травмы
// ═══════════════════════════════════════════════════════════════════════════════

@compute @workgroup_size(256)
fn cs_propagate_trauma(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    
    if idx >= params.num_voxels {
        return;
    }
    
    var voxel = voxels[idx];
    
    if (voxel.flags & FLAG_ALIVE) == 0u {
        return;
    }
    
    // Если воксель травмирован, распространить часть травмы соседям
    if (voxel.flags & FLAG_TRAUMATIZED) != 0u && voxel.trauma_level > 0.3 {
        let spread_amount = voxel.trauma_level * 0.1 * params.delta_time;
        
        // Распространить на соседа 0
        let conn0 = unpack_u16(voxel.connections_01, 0u);
        if conn0 < params.num_voxels && conn0 != 0xFFFFu {
            voxels[conn0].trauma_level = min(voxels[conn0].trauma_level + spread_amount, 1.0);
        }
    }
    
    voxels[idx] = voxel;
}
