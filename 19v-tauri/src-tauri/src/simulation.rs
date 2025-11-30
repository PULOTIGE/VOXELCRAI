use std::{collections::VecDeque, time::{Duration, Instant}};

use chrono::{DateTime, Utc};
use glam::{Mat3, Vec2, Vec3};
use parking_lot::Mutex;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use smallvec::SmallVec;
use uuid::Uuid;

use crate::light_pattern::LightPatternSnapshot;

pub const TOTAL_NUCLEOTIDES: usize = 62_000_000;
const SAMPLE_COUNT: usize = 4_096;
const ACTIVE_BYTES: usize = 256;
const WARM_BYTES: usize = 64;
const SLEEP_BYTES: usize = 32;
const BEACON_BYTES: usize = 16;
const MAX_LOG_ENTRIES: usize = 512;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum ConceptOrigin {
    DuckDuckGo,
    FileDrop,
    Manual,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: Uuid,
    pub label: String,
    pub digest: String,
    pub origin: ConceptOrigin,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub id: Uuid,
    pub level: LogLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct PointParticle {
    pub id: u64,
    pub position: Vec3,
    pub energy: f32,
    pub emotion: Emotion,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Emotion {
    pub valence: f32,
    pub arousal: f32,
    pub dominance: f32,
}

#[derive(Clone)]
pub struct SimulationSnapshot {
    pub particles: Vec<PointParticle>,
    pub updates_processed: u128,
    pub tick_time: Duration,
    pub average_energy: f32,
    pub concept_count: usize,
    pub log: Vec<LogEntry>,
}

#[derive(Clone)]
pub struct CameraState {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.35,
            distance: 4.0,
        }
    }
}

impl CameraState {
    pub fn orbit(&mut self, delta: Vec2) {
        self.yaw += delta.x * 0.01;
        self.pitch = (self.pitch + delta.y * 0.01).clamp(-1.2, 1.2);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance - delta * 0.1).clamp(1.0, 12.0);
    }

    pub fn project(&self, point: Vec3, viewport: egui::Rect) -> Option<(egui::Pos2, f32)> {
        let rotation = Mat3::from_rotation_y(self.yaw) * Mat3::from_rotation_x(self.pitch);
        let transformed = rotation * point;
        let depth = transformed.z + self.distance;
        if depth <= 0.05 {
            return None;
        }
        let scale = (viewport.width().min(viewport.height())) * 0.35 / depth;
        let pos = egui::pos2(
            viewport.center().x + transformed.x * scale,
            viewport.center().y - transformed.y * scale,
        );
        Some((pos, depth))
    }
}

#[derive(Clone)]
pub struct SharedSimulation {
    inner: std::sync::Arc<SimulationInner>,
}

struct SimulationInner {
    bank: Mutex<NucleotideBank>,
    concepts: Mutex<Vec<Concept>>,
    log: Mutex<VecDeque<LogEntry>>,
}

impl SharedSimulation {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Arc::new(SimulationInner {
                bank: Mutex::new(NucleotideBank::new()),
                concepts: Mutex::new(Vec::new()),
                log: Mutex::new(VecDeque::with_capacity(MAX_LOG_ENTRIES)),
            }),
        }
    }

    pub fn clone_arc(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    pub fn tick(&self, pattern: &LightPatternSnapshot) -> SimulationSnapshot {
        let start = Instant::now();
        let mut bank = self.inner.bank.lock();
        let particles = bank.tick(pattern);
        let tick_time = start.elapsed();
        let log = {
            let mut guard = self.inner.log.lock();
            guard.make_contiguous();
            guard.iter().cloned().collect::<Vec<_>>()
        };
        SimulationSnapshot {
            particles,
            updates_processed: bank.updates_processed,
            tick_time,
            average_energy: bank.average_energy,
            concept_count: self.inner.concepts.lock().len(),
            log,
        }
    }

    pub fn ingest_concepts(&self, label: &str, origin: ConceptOrigin, payload: &[u8]) {
        let digest = format!("{:x}", Sha256::digest(payload));
        let concept = Concept {
            id: Uuid::new_v4(),
            label: label.to_string(),
            digest,
            origin,
            timestamp: Utc::now(),
        };
        self.inner.concepts.lock().push(concept.clone());
        self.log(LogLevel::Info, format!("Запомнен концепт {}", concept.label));
        self.inner.bank.lock().imprint_concept(&concept);
    }

    pub fn log(&self, level: LogLevel, message: impl Into<String>) {
        let mut log = self.inner.log.lock();
        if log.len() >= MAX_LOG_ENTRIES {
            log.pop_front();
        }
        log.push_back(LogEntry {
            id: Uuid::new_v4(),
            level,
            message: message.into(),
            timestamp: Utc::now(),
        });
    }
}

#[derive(Clone)]
struct Nucleotide {
    id: u64,
    payload: NucleotidePayload,
    blueprint: GeneticBlueprint,
    emotion: Emotion,
    energy: f32,
    position: Vec3,
}

#[derive(Clone)]
struct NucleotidePayload {
    active: [u8; ACTIVE_BYTES],
    warm: [u8; WARM_BYTES],
    sleep: [u8; SLEEP_BYTES],
    beacons: [u8; BEACON_BYTES],
}

#[derive(Clone)]
struct GeneticBlueprint {
    sequence: SmallVec<[Base; 128]>,
    epigenetics: [f32; 4],
    quantum_noise: f32,
    chromatin_density: f32,
}

#[derive(Clone, Copy)]
enum Base {
    A,
    T,
    G,
    C,
}

impl Base {
    fn random(rng: &mut ChaCha20Rng) -> Self {
        match rng.gen_range(0..4) {
            0 => Base::A,
            1 => Base::T,
            2 => Base::G,
            _ => Base::C,
        }
    }

    fn as_char(&self) -> char {
        match self {
            Base::A => 'A',
            Base::T => 'T',
            Base::G => 'G',
            Base::C => 'C',
        }
    }
}

pub struct NucleotideBank {
    rng: ChaCha20Rng,
    sample: Vec<Nucleotide>,
    pub updates_processed: u128,
    pub average_energy: f32,
    concepts_cache: Vec<String>,
}

impl NucleotideBank {
    fn new() -> Self {
        let mut rng = ChaCha20Rng::from_seed([42; 32]);
        let mut sample = Vec::with_capacity(SAMPLE_COUNT);
        for idx in 0..SAMPLE_COUNT {
            sample.push(Self::spawn(idx as u64, &mut rng));
        }
        Self {
            rng,
            sample,
            updates_processed: 0,
            average_energy: 0.5,
            concepts_cache: Vec::new(),
        }
    }

    fn spawn(id: u64, rng: &mut ChaCha20Rng) -> Nucleotide {
        let mut payload = NucleotidePayload {
            active: [0; ACTIVE_BYTES],
            warm: [0; WARM_BYTES],
            sleep: [0; SLEEP_BYTES],
            beacons: [0; BEACON_BYTES],
        };
        rng.fill(&mut payload.active);
        rng.fill(&mut payload.warm);
        rng.fill(&mut payload.sleep);
        rng.fill(&mut payload.beacons);

        let mut sequence = SmallVec::<[Base; 128]>::new();
        for _ in 0..128 {
            sequence.push(Base::random(rng));
        }

        Nucleotide {
            id,
            payload,
            blueprint: GeneticBlueprint {
                sequence,
                epigenetics: [rng.gen(), rng.gen(), rng.gen(), rng.gen()],
                quantum_noise: rng.gen(),
                chromatin_density: rng.gen(),
            },
            emotion: Emotion {
                valence: rng.gen_range(-1.0..1.0),
                arousal: rng.gen_range(0.0..1.0),
                dominance: rng.gen_range(-0.4..0.8),
            },
            energy: rng.gen_range(0.2..0.9),
            position: Vec3::new(
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
            ),
        }
    }

    fn tick(&mut self, pattern: &LightPatternSnapshot) -> Vec<PointParticle> {
        self.updates_processed = self.updates_processed.saturating_add(TOTAL_NUCLEOTIDES as u128);
        let mut total_energy = 0.0;
        let mut particles = Vec::with_capacity(self.sample.len());
        let field_len = pattern.values.len().max(1);
        for (idx, nucleotide) in self.sample.iter_mut().enumerate() {
            let field = pattern.values[idx % field_len];
            nucleotide.energy = (nucleotide.energy * 0.8) + field * 0.2;
            nucleotide.emotion.valence = (nucleotide.emotion.valence * 0.9)
                + (field * 2.0 - 1.0) * 0.1;
            nucleotide.emotion.arousal = (nucleotide.emotion.arousal * 0.7) + 0.3 * field;
            nucleotide.emotion.dominance = (nucleotide.emotion.dominance * 0.95)
                + self.rng.gen_range(-0.02..0.02);
            nucleotide.position += Vec3::new(
                self.rng.gen_range(-0.01..0.01),
                self.rng.gen_range(-0.01..0.01),
                self.rng.gen_range(-0.01..0.01),
            );
            nucleotide.position = nucleotide.position.clamp(Vec3::splat(-2.0), Vec3::splat(2.0));

            total_energy += nucleotide.energy;
            particles.push(PointParticle {
                id: nucleotide.id,
                position: nucleotide.position,
                energy: nucleotide.energy,
                emotion: nucleotide.emotion,
            });
        }
        self.average_energy = total_energy / self.sample.len() as f32;
        particles
    }

    fn imprint_concept(&mut self, concept: &Concept) {
        self.concepts_cache.push(concept.label.clone());
        if self.concepts_cache.len() > 64 {
            self.concepts_cache.remove(0);
        }
        for (slot, chunk) in self.sample.iter_mut().take(self.concepts_cache.len()).enumerate() {
            let imprint = self.concepts_cache[slot].as_bytes();
            for (idx, byte) in imprint.iter().enumerate().take(chunk.payload.active.len()) {
                chunk.payload.active[idx] ^= byte;
            }
        }
    }

    pub fn sequence_fragment(&self, idx: usize) -> String {
        let nucleotide = &self.sample[idx % self.sample.len()];
        nucleotide
            .blueprint
            .sequence
            .iter()
            .take(32)
            .map(Base::as_char)
            .collect()
    }
}

impl SharedSimulation {
    pub fn concepts(&self) -> Vec<Concept> {
        self.inner.concepts.lock().clone()
    }

    pub fn sequence_fragment(&self, idx: usize) -> String {
        self.inner.bank.lock().sequence_fragment(idx)
    }
}
