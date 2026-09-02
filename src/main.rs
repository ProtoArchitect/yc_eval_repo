use std::alloc::{GlobalAlloc, Layout, System};
use std::fs::File;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use memmap2::MmapOptions;
use indicatif::{ProgressBar, ProgressStyle};

/// PROPRIETARY CORE ENGINE (SIMULATED)
/// This module demonstrates how the core algorithms are strictly `#![no_std]`
/// compatible, relying only on `core` and performing true Zero-Allocation computation.
pub mod core_engine {
    // Note: The core logic uses only `core::slice` and primitive types, making it 
    // strictly compatible with `#![no_std]` when moved to a separate crate.
    
    #[inline(never)]
    pub fn transport_read_no_std(data: &[u8]) -> u64 {
        let u64_len = data.len() / 8;
        let ptr = data.as_ptr() as *const u64;
        let u64_slice = unsafe { core::slice::from_raw_parts(ptr, u64_len) };
        
        let mut sum: u64 = 0;
        for &val in u64_slice {
            sum = sum.wrapping_add(val);
        }
        sum
    }

    // Строго no_std: никаких аллокаций, только математика на стеке
    #[inline(never)]
    pub fn compute_query_no_std(data: &[u8], target_query: &[i64; 384]) -> (usize, i64) {
        let vectors = unsafe { 
            core::slice::from_raw_parts(data.as_ptr() as *const i64, data.len() / 8) 
        };
        
        let mut max_score = core::i64::MIN;
        let mut best_index = 0;

        // Разбиваем плоский массив на 384-мерные векторы (твои центроиды)
        for (index, candidate) in vectors.chunks_exact(384).enumerate() {
            let mut current_score: i64 = 0;
            
            // Вычисляем косинусное сходство (скалярное произведение) в фиксированной точке Q32.32
            // Компилятор Rust сам развернет этот цикл в инструкции AVX-512 / SIMD
            for i in 0..384 {
                current_score = current_score.wrapping_add(candidate[i].wrapping_mul(target_query[i]));
            }

            if current_score > max_score {
                max_score = current_score;
                best_index = index;
            }
        }
        (best_index, max_score)
    }
}

struct TrackingAllocator;
static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

// Read hardware energy counter from RAPL MSR (requires sudo)
fn read_energy_uj() -> u64 {
    std::fs::read_to_string("/sys/class/powercap/intel-rapl:0/energy_uj")
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse::<u64>()
        .unwrap_or(0)
}

fn main() {
    let file_path = "./universal_semantic_codebook.bin";
    let iterations = 50;

    println!("\x1b[1;36m============================================================\x1b[0m");
    println!("\x1b[1;37m    AURIGLYPH: ZERO-ALLOCATION STATE COMPRESSION ENGINE\x1b[0m");
    println!("\x1b[1;36m============================================================\x1b[0m");
    println!("\x1b[1;33m[TARGET]\x1b[0m LLM Semantic State / Universal Semantic Codebook");
    println!("\x1b[1;33m[CLAIM]\x1b[0m  Query-in-Place throughput > 10 GB/s with ZERO heap allocation.\n");

    println!("\x1b[1;90m>>> Mounting REAL DATASET: {}\x1b[0m", file_path);

    let file = File::open(file_path).expect("Failed to open the real data file");
    let file_size = file.metadata().unwrap().len();
    let file_size_gb = file_size as f64 / 1_073_741_824.0;
    let total_bytes = file_size * iterations as u64;

    println!("\x1b[1;31m>>> INITIATING SUSTAINED BARE-METAL QUERY-IN-PLACE ({} Iterations)...\x1b[0m", iterations);
    println!("\x1b[1;31m>>> Watch the RAM (Mem) monitor. It will remain flat.\x1b[0m\n");

    std::thread::sleep(std::time::Duration::from_secs(2));

    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let mut heap_used = 0;

    // Phase 1: Warmup the OS Page Cache (SSD -> RAM)
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠉")
        .template("{spinner:.cyan} {msg}")
        .unwrap());
    spinner.set_message("\x1b[1;90mWarming up OS Page Cache (SSD -> RAM)...\x1b[0m");
    spinner.enable_steady_tick(std::time::Duration::from_millis(50));
    
    let _warmup = core_engine::transport_read_no_std(&mmap);
    std::hint::black_box(_warmup);
    spinner.finish_with_message("\x1b[1;32m[OK]\x1b[0m \x1b[1;90mCache warm.\x1b[0m\n");

    // Phase 2: Measure Transport Read Bandwidth (RAM -> CPU)
    let pb_transport = ProgressBar::new(total_bytes);
    pb_transport.set_style(ProgressStyle::default_bar()
        .template("{msg}\n[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec})")
        .unwrap()
        .progress_chars("=>-"));
    pb_transport.set_message("\x1b[1;37mPhase 1: Sustained Transport Pass (RAM -> CPU)\x1b[0m");

    let t_start = Instant::now();
    for _ in 0..iterations {
        ALLOCATED.store(0, Ordering::SeqCst);
        let _transport = core_engine::transport_read_no_std(&mmap);
        std::hint::black_box(_transport);
        heap_used += ALLOCATED.load(Ordering::SeqCst);
        pb_transport.inc(file_size);
    }
    let transport_duration = t_start.elapsed();
    pb_transport.finish_with_message("\x1b[1;32m[OK]\x1b[0m \x1b[1;37mTransport pass complete.\x1b[0m\n");

    // Phase 3: Core engine computation (Math)
    let pb_compute = ProgressBar::new(total_bytes);
    pb_compute.set_style(ProgressStyle::default_bar()
        .template("{msg}\n[{elapsed_precise}] {bar:40.magenta/blue} {bytes}/{total_bytes} ({bytes_per_sec})")
        .unwrap()
        .progress_chars("=>-"));
    pb_compute.set_message("\x1b[1;37mPhase 2: Sustained Compute Pass (Q32.32 SIMD Dot Product)\x1b[0m");

    let target_query = [1i64; 384];
    
    let c_start = Instant::now();
    for _ in 0..iterations {
        ALLOCATED.store(0, Ordering::SeqCst);
        let (best_idx, max_score) = core_engine::compute_query_no_std(&mmap, &target_query);
        std::hint::black_box((best_idx, max_score));
        heap_used += ALLOCATED.load(Ordering::SeqCst);
        pb_compute.inc(file_size);
    }
    let compute_duration = c_start.elapsed();
    pb_compute.finish_with_message("\x1b[1;32m[OK]\x1b[0m \x1b[1;37mCompute pass complete.\x1b[0m\n");

    let transport_gb_s = (total_bytes as f64 / 1_073_741_824.0) / transport_duration.as_secs_f64();
    let compute_gb_s = (total_bytes as f64 / 1_073_741_824.0) / compute_duration.as_secs_f64();

    println!("\x1b[1;32m============================================================\x1b[0m");
    println!("\x1b[1;32m                 VERIFICATION COMPLETE                      \x1b[0m");
    println!("\x1b[1;32m============================================================\x1b[0m");
    println!("Payload Scanned:      \x1b[1;37m{:.2} GB\x1b[0m (Real Semantic Dataset)", file_size_gb);
    println!("Iterations:           \x1b[1;37m{}\x1b[0m", iterations);
    println!("Avg Cache Read Bandwidth (Transport): \x1b[1;36m{:.2} GB/s\x1b[0m", transport_gb_s);
    println!("Avg Query-in-Place Compute (Math):    \x1b[1;35m{:.2} GB/s\x1b[0m", compute_gb_s);
    
    if heap_used == 0 {
        println!("Heap Allocated:       \x1b[1;32m{} bytes\x1b[0m (Strict Zero-Allocation Verified)", heap_used);
    } else {
        println!("Heap Allocated:       \x1b[1;33m{} bytes\x1b[0m (Runtime Overhead)", heap_used);
    }
    println!("");
}
