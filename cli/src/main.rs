mod export;
mod backend;
mod json;
mod save;
mod tables_gen {
    include!("tables_gen.rs");
}
mod ui;

use backend::{backend_progress, default_threads, load_backend, Backend};
use save::{json_f64, json_i64, merge_into_save, save_path, unit_price};
use std::ffi::c_void;
use std::fs;
use std::io::IsTerminal;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tables_gen::{PACKMAP, SHOP_LEVEL_FALLBACK};
use ui::{human, money, render_bar, term_width};

#[cfg(windows)]
const CPU_LIB: &str = "libpackopen_cpu.dll";
#[cfg(windows)]
const GPU_LIB: &str = "libpackopen_gpu.dll";
#[cfg(not(windows))]
const CPU_LIB: &str = "libpackopen_cpu.so";
#[cfg(not(windows))]
const GPU_LIB: &str = "libpackopen_gpu.so";

fn packmap(eitem: i32) -> Option<(i32, i32)> {
    if eitem == 129 {
        return Some((15, 7));
    }
    PACKMAP.iter().find(|p| p.0 == eitem).map(|p| (p.1, p.2))
}

fn status() {
    let sp = save_path(None);
    let txt = fs::read_to_string(&sp).unwrap_or_default();
    let idx = json_i64(&txt, "\"m_SaveIndex\"").unwrap_or(-1);
    let level = json_i64(&txt, "\"m_ShopLevel\"").unwrap_or(-1);
    let coins = json_f64(&txt, "\"m_CoinAmountDouble\"").unwrap_or(0.0);
    println!("save: {}", sp);
    println!("  slot {} | level {} | {}", idx, level, money(coins));
}

fn list_packs() {
    println!("eitem  collection  exp  name-hint");
    for (e, c, x) in PACKMAP {
        let name = match (c, x) {
            (0, 0) => "Basic",
            (1, 0) => "Rare",
            (2, 0) => "Epic",
            (3, 0) => "Legendary",
            (4, 1) => "DestinyBasic",
            (5, 1) => "DestinyRare",
            (6, 1) => "DestinyEpic",
            (7, 1) => "DestinyLegendary",
            (8, 2) => "Ghost",
            _ => "?",
        };
        println!("{:>5}  {:>10}  {:>3}  {}", e, c, x, name);
    }
}

fn generate_random_seed() -> u64 {
    use std::hash::{BuildHasher, Hasher, RandomState};
    let h1 = RandomState::new().build_hasher().finish();
    let h2 = RandomState::new().build_hasher().finish();
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    h1 ^ h2.rotate_left(32) ^ t
}

fn parse_seed_str(s: &str) -> Option<u64> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u64>().ok()
    }
}

fn resolve_seed_val(custom_seed: Option<u64>) -> (u64, bool) {
    if let Some(s) = custom_seed {
        (s, true)
    } else if let Ok(env_s) = std::env::var("PACKSIM_SEED") {
        if let Some(s) = parse_seed_str(&env_s) {
            (s, true)
        } else {
            (generate_random_seed(), false)
        }
    } else {
        (generate_random_seed(), false)
    }
}

fn peek_ckpt_seed(path: &str) -> Option<u64> {
    use std::io::Read;
    let mut f = fs::File::open(path).ok()?;
    let mut header = [0u8; 8 + 4 + 4 + 4 + 8 + 8 + 8 + 8 + 8]; // up to seed (60 bytes)
    f.read_exact(&mut header).ok()?;
    if &header[0..8] == b"PKCKPT03" {
        let seed = u64::from_le_bytes(header[52..60].try_into().ok()?);
        Some(seed)
    } else if &header[0..8] == b"PKCKPT02" {
        Some(0x12345678abcdef01u64)
    } else {
        None
    }
}

fn load_available_backends(backend: &str) -> (Option<Backend>, Option<Backend>, bool) {
    let cpu_lib = if backend == "cpu" || backend == "auto" || backend == "gpu" {
        load_backend(CPU_LIB, "CPU")
    } else {
        None
    };
    let gpu_lib = if backend == "gpu" || backend == "auto" {
        load_backend(GPU_LIB, "GPU")
    } else {
        None
    };
    let use_gpu = match backend {
        "gpu" => gpu_lib
            .as_ref()
            .map(|g| unsafe { (g.probe)() == 1 })
            .unwrap_or(false),
        "cpu" => false,
        _ => {
            if let Some(g) = &gpu_lib {
                unsafe { (g.probe)() == 1 }
            } else {
                false
            }
        }
    };
    (cpu_lib, gpu_lib, use_gpu)
}

fn usage() -> &'static str {
    "packopener-cli: simulate pack openings + merge results into the game save.

Usage:
  packopener-cli <eitem> [count] [options]    simulate + merge (--dry-run to skip)
  packopener-cli list                         show pack types
  packopener-cli merge [file] [options]       merge an existing results file
  packopener-cli export <file> --format <f> [opts] export results file
  packopener-cli status                       show save state
  packopener-cli benchmark [packs] [options]  measure backend throughput
  packopener-cli help                         this text

Options (simulate):
  <eitem>      pack type id (see `list`; 129 = Ascension)
  [count]      packs to open; default: as many as affordable with current coins
  --threads N  CPU worker threads (default: parallelism x2)
  --backend M  cpu | gpu | auto (default: auto)
  --slot N     save slot to merge into (default: 0)
  --save F     save file (explicit)
  --seed N     PRNG root seed (default: random; pass a number/hex for a static seed)
  --results F  results file (default: results.simple)
  --sold F     sold file (default: beside the results file)
  --width N    terminal width override for the progress bar
  --apply      write the save (--dry-run to skip)
  --debug      verbose progress diagnostics

Options (merge):
  [file]       results file (default: results.simple)
  --results F  results file (explicit)
  --sold F     sold file (default: results_sold.simple)
  --save F     save file (explicit)
  --apply      write the save (--dry-run to skip)

Options (export):
  <file>       results file
  --format F   csv | json
  -o, --out F  output file (default: stdout)

Options (benchmark):
  [packs]      packs to simulate (default: 10000000)
  --backend M  cpu | gpu | auto
  --threads N  CPU worker threads
  --seed N     PRNG root seed (default: random)

Long sims checkpoint beside the results file (rerun to resume; no merge until the run completes).
Merge deducts pack cost (count x unit price) from coins, then adds overflow-sold cash.
Every merged slot is capped at 2147483647; excess is auto-sold at market price."
}

fn cmd_merge(args: &[String]) {
    let mut results = "results.simple".to_string();
    let mut sold = "results_sold.simple".to_string();
    let mut save: Option<String> = None;
    let mut apply = true;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" | "--dry" => apply = false,
            "--apply" => apply = true,
            "--results" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    results = v.clone();
                }
            }
            "--sold" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    sold = v.clone();
                }
            }
            "--save" | "--slot-file" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    save = Some(v.clone());
                }
            }
            s if !s.starts_with("--") => {
                if s.ends_with(".json") {
                    save = Some(s.to_string());
                } else {
                    results = s.to_string();
                }
            }
            _ => {}
        }
        i += 1;
    }
    let sp = save
        .map(|s| s.trim_end_matches(['/', '\\']).to_string())
        .unwrap_or_else(|| save_path(None));
    let results_text = match fs::read_to_string(&results) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("merge FAILED: cannot read {}: {}", results, e);
            std::process::exit(1);
        }
    };
    let sold_text = if !sold.is_empty() {
        fs::read_to_string(&sold).unwrap_or_default()
    } else {
        String::new()
    };
    eprintln!("note: merging a file does not deduct pack cost (unknown)");
    match merge_into_save(&sp, apply, &results_text, &sold_text, 0.0) {
        Ok(()) => eprintln!("merge ok"),
        Err(e) => {
            eprintln!("merge FAILED: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_export(args: &[String]) {
    if args.is_empty() {
        eprintln!("error: missing results file for export (see: packopener-cli help)");
        std::process::exit(2);
    }
    let mut input_file = String::new();
    let mut format = String::new();
    let mut out: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    format = v.clone();
                }
            }
            "-o" | "--out" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    out = Some(v.clone());
                }
            }
            s if !s.starts_with("--") => {
                input_file = s.to_string();
            }
            s => {
                eprintln!("error: unknown argument: {} (see: packopener-cli help)", s);
                std::process::exit(2);
            }
        }
        i += 1;
    }
    if input_file.is_empty() {
        eprintln!("error: missing results file for export");
        std::process::exit(2);
    }
    if format.is_empty() {
        eprintln!("error: --format <csv|json> is required");
        std::process::exit(2);
    }
    if let Err(e) = export::export_results(&input_file, out.as_deref(), &format) {
        if e.contains("Broken pipe") {
            std::process::exit(0);
        }
        eprintln!("export failed: {}", e);
        std::process::exit(1);
    }
    std::process::exit(0);
}

fn cmd_benchmark(args: &[String]) {
    let mut packs = 10_000_000u64;
    let mut threads = default_threads();
    let mut backend = "auto".to_string();
    let mut custom_seed: Option<u64> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--threads" | "-t" => {
                i += 1;
                threads = match args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    Some(v) => v.clamp(1, 256),
                    None => {
                        eprintln!("error: --threads needs a number");
                        std::process::exit(2);
                    }
                };
            }
            "--backend" => {
                i += 1;
                backend = match args.get(i).cloned() {
                    Some(v) if v == "cpu" || v == "gpu" || v == "auto" => v,
                    Some(v) => {
                        eprintln!("error: --backend must be cpu|gpu|auto (got {})", v);
                        std::process::exit(2);
                    }
                    None => {
                        eprintln!("error: --backend needs a value");
                        std::process::exit(2);
                    }
                };
            }
            "--seed" => {
                i += 1;
                custom_seed = match args.get(i).and_then(|s| parse_seed_str(s)) {
                    Some(v) => Some(v),
                    None => {
                        eprintln!("error: --seed needs a valid 64-bit integer (e.g. 12345 or 0x12345678abcdef01)");
                        std::process::exit(2);
                    }
                };
            }
            s => match s.parse::<u64>() {
                Ok(v) => packs = v,
                Err(_) => {
                    eprintln!("error: unknown argument or bad pack count: {} (see: packopener-cli help)", s);
                    std::process::exit(2);
                }
            },
        }
        i += 1;
    }
    let (cpu_lib, gpu_lib, use_gpu) = load_available_backends(&backend);
    let picked: Option<&Backend> = if use_gpu {
        gpu_lib.as_ref()
    } else {
        cpu_lib.as_ref()
    };
    let b = match picked {
        Some(b) => b,
        None => {
            eprintln!("benchmark: no backend found ({} / {})", CPU_LIB, GPU_LIB);
            std::process::exit(1);
        }
    };
    let par = if use_gpu { 0 } else { threads as i32 };
    let results_c = std::ffi::CString::new("").unwrap();
    let sold_c = std::ffi::CString::new("").unwrap();
    let mut seconds = 0f64;
    let (seed, is_static) = resolve_seed_val(custom_seed);
    std::env::set_var("PACKSIM_SEED", seed.to_string());
    if is_static {
        eprintln!("seed: 0x{:016x} (static)", seed);
    } else {
        eprintln!("seed: 0x{:016x} (random; pass --seed to reproduce)", seed);
    }
    let rc = if let Some(f) = b.sim_seed_ckpt {
        unsafe {
            f(
                0,
                packs,
                par,
                SHOP_LEVEL_FALLBACK,
                results_c.as_ptr() as *const u8,
                sold_c.as_ptr() as *const u8,
                std::ptr::null(),
                &mut seconds,
                None,
                std::ptr::null_mut(),
                seed,
            )
        }
    } else if let Some(f) = b.sim_ckpt {
        unsafe {
            f(
                0,
                packs,
                par,
                SHOP_LEVEL_FALLBACK,
                results_c.as_ptr() as *const u8,
                sold_c.as_ptr() as *const u8,
                std::ptr::null(),
                &mut seconds,
                None,
                std::ptr::null_mut(),
            )
        }
    } else if let Some(f) = b.sim_seed {
        unsafe {
            f(
                0,
                packs,
                par,
                SHOP_LEVEL_FALLBACK,
                results_c.as_ptr() as *const u8,
                sold_c.as_ptr() as *const u8,
                &mut seconds,
                seed,
            )
        }
    } else {
        unsafe {
            (b.sim)(
                0,
                packs,
                par,
                SHOP_LEVEL_FALLBACK,
                results_c.as_ptr() as *const u8,
                sold_c.as_ptr() as *const u8,
                &mut seconds,
            )
        }
    };
    if rc != 0 {
        eprintln!("benchmark: {} backend failed (rc={})", b.label, rc);
        std::process::exit(1);
    }
    let par_desc = if use_gpu {
        "auto blocks".to_string()
    } else {
        format!("{} threads", threads)
    };
    eprintln!(
        "{} packs, {} backend ({}), {:.2}s = {} packs/s",
        human(packs as f64),
        b.label,
        par_desc,
        seconds,
        human(packs as f64 / seconds)
    );
}

fn cmd_simulate(eitem_str: &str, args: &[String]) {
    let eitem: i32 = match eitem_str.parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("unknown command or pack id: {}", eitem_str);
            std::process::exit(2);
        }
    };
    let mut threads = default_threads();
    let mut slot: Option<usize> = Some(0);
    let mut save_override: Option<String> = None;
    let mut apply = true;
    let mut count: Option<u64> = None;
    let mut results_file = "results.simple".to_string();
    let mut sold_flag: Option<String> = None;
    let mut width_override: Option<usize> = None;
    let mut debug = false;
    let mut backend = "auto".to_string();
    let mut custom_seed: Option<u64> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--threads" | "-t" => {
                i += 1;
                threads = match args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    Some(v) => v.clamp(1, 256),
                    None => {
                        eprintln!("error: --threads needs a number");
                        std::process::exit(2);
                    }
                };
            }
            "--slot" => {
                i += 1;
                slot = Some(match args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    Some(v) => v,
                    None => {
                        eprintln!("error: --slot needs a number");
                        std::process::exit(2);
                    }
                });
            }
            "--save" | "--slot-file" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    save_override = Some(v.clone());
                } else {
                    eprintln!("error: --save needs a file path");
                    std::process::exit(2);
                }
            }
            "--results" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    results_file = v.clone();
                }
            }
            "--sold" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    sold_flag = Some(v.clone());
                }
            }
            "--seed" => {
                i += 1;
                custom_seed = match args.get(i).and_then(|s| parse_seed_str(s)) {
                    Some(v) => Some(v),
                    None => {
                        eprintln!("error: --seed needs a valid 64-bit integer (e.g. 12345 or 0x12345678abcdef01)");
                        std::process::exit(2);
                    }
                };
            }
            "--width" | "-w" => {
                i += 1;
                width_override =
                    Some(match args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                        Some(v) => v,
                        None => {
                            eprintln!("error: --width needs a number");
                            std::process::exit(2);
                        }
                    });
            }
            "--debug" => debug = true,
            "--backend" => {
                i += 1;
                backend = match args.get(i).cloned() {
                    Some(v) if v == "cpu" || v == "gpu" || v == "auto" => v,
                    Some(v) => {
                        eprintln!("error: --backend must be cpu|gpu|auto (got {})", v);
                        std::process::exit(2);
                    }
                    None => {
                        eprintln!("error: --backend needs a value");
                        std::process::exit(2);
                    }
                };
            }
            "--dry-run" | "--dry" => apply = false,
            "--apply" => apply = true,
            s => {
                count = Some(match s.parse::<u64>() {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("error: unknown argument or bad count: {} (see: packopener-cli help)", s);
                        std::process::exit(2);
                    }
                });
            }
        }
        i += 1;
    }
    let sp = save_override
        .map(|s| s.trim_end_matches(['/', '\\']).to_string())
        .unwrap_or_else(|| save_path(slot));
    let have_save = fs::metadata(&sp).is_ok();
    let txt = if have_save {
        fs::read_to_string(&sp).unwrap_or_default()
    } else {
        String::new()
    };
    let coins = json_f64(&txt, "\"m_CoinAmountDouble\"").unwrap_or(0.0);
    let unit = unit_price(&txt, eitem).unwrap_or(1.61);
    let shop =
        json_i64(&txt, "\"m_ShopLevel\"").unwrap_or(SHOP_LEVEL_FALLBACK as i64) as i32;
    let affordable = (coins / unit) as u64;
    let requested = count.unwrap_or(affordable);
    let n = if !have_save || !apply {
        requested
    } else {
        requested.min(affordable)
    };
    let (collection, exp_id) = packmap(eitem).expect("unknown pack type");
    eprintln!(
        "eitem {} (collection {}, exp {}) | {}/pack | {} coins -> {} packs {}",
        eitem,
        collection,
        exp_id,
        money(unit),
        money(coins),
        human(n as f64),
        if count.is_none() {
            "(affordable)"
        } else if n < requested {
            "(capped to affordable)"
        } else {
            ""
        }
    );
    if n == 0 {
        eprintln!(
            "nothing to do: 0 packs (can't afford any at {})",
            money(unit)
        );
        return;
    }

    let pack_cost = n as f64 * unit;
    let results_stem = results_file
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(results_file.as_str())
        .to_string();
    let sold_file = sold_flag.unwrap_or_else(|| format!("{}_sold.simple", results_stem));
    let ckpt_file = format!("{}.ckpt", results_stem);

    let ckpt_seed = peek_ckpt_seed(&ckpt_file);
    let (seed, is_static, is_resumed) = match (custom_seed, ckpt_seed) {
        (Some(s), Some(cs)) => {
            if s != cs {
                eprintln!("warning: checkpoint {} was created with seed 0x{:016x}, but --seed 0x{:016x} was requested; resuming with checkpoint's original seed", ckpt_file, cs, s);
            }
            (cs, true, true)
        }
        (None, Some(cs)) => (cs, false, true),
        (Some(s), None) => (s, true, false),
        (None, None) => {
            let (s, st) = resolve_seed_val(None);
            (s, st, false)
        }
    };

    std::env::set_var("PACKSIM_SEED", seed.to_string());
    if is_resumed {
        eprintln!("resuming checkpoint: {} (seed: 0x{:016x})", ckpt_file, seed);
    } else if is_static {
        eprintln!("seed: 0x{:016x} (static)", seed);
    } else {
        eprintln!("seed: 0x{:016x} (random; pass --seed to reproduce)", seed);
    }

    let tty = std::io::stderr().is_terminal();
    let termw = term_width(width_override, debug);

    let (cpu_lib, gpu_lib, use_gpu) = load_available_backends(&backend);
    if backend == "gpu" && !use_gpu {
        eprintln!(
            "--backend gpu unavailable (no {} or no CUDA device), falling back to CPU",
            GPU_LIB
        );
    }
    if backend == "cpu" && cpu_lib.is_none() {
        eprintln!("fatal: {} not found", CPU_LIB);
        std::process::exit(1);
    }

    let mut backend_ok = false;
    let mut order: Vec<(bool, &Backend)> = Vec::new();
    if use_gpu {
        if let Some(g) = gpu_lib.as_ref() {
            order.push((true, g));
        }
    }
    if let Some(c) = cpu_lib.as_ref() {
        order.push((false, c));
    }
    for (is_gpu, b) in &order {
        let par = if *is_gpu { 0 } else { threads as i32 };
        let par_desc = if *is_gpu {
            "auto blocks".to_string()
        } else {
            format!("{} threads", threads)
        };
        eprintln!("backend: {} ({})", b.label, par_desc);
        let results_c = std::ffi::CString::new(results_file.as_str()).unwrap();
        let sold_c = std::ffi::CString::new(sold_file.as_str()).unwrap();
        let ckpt_c = std::ffi::CString::new(ckpt_file.as_str()).unwrap();
        let mut seconds = 0f64;
        if b.sim_ckpt.is_some() {
            let done = Arc::new(AtomicU64::new(0));
            let finished = Arc::new(AtomicBool::new(false));
            let bar_done = Arc::new(AtomicBool::new(false));
            std::thread::scope(|s| {
                {
                    let done = done.clone();
                    let finished = finished.clone();
                    let bar_done = bar_done.clone();
                    s.spawn(move || {
                        let start = Instant::now();
                        let mut last_total = 0u64;
                        let mut last_total_t = start;
                        let mut last_display = 0u64;
                        let mut last_display_t = start;
                        let mut rate = 0f64;
                        let mut last_log = start;
                        let mut first = true;
                        loop {
                            std::thread::sleep(Duration::from_millis(50));
                            let total = done.load(Ordering::Relaxed);
                            if first && total == 0 {
                                if finished.load(Ordering::Relaxed) {
                                    bar_done.store(true, Ordering::Relaxed);
                                    break;
                                }
                                continue;
                            }
                            if first {
                                first = false;
                                last_total_t = Instant::now();
                                last_display_t = last_total_t;
                                last_log = last_total_t;
                            }
                            let now = Instant::now();
                            if total > last_total {
                                let dt = (now - last_total_t).as_secs_f64().max(1e-6);
                                let inst = (total - last_total) as f64 / dt;
                                rate = if rate == 0.0 {
                                    inst
                                } else {
                                    rate * 0.5 + inst * 0.5
                                };
                                last_total = total;
                                last_total_t = now;
                            }
                            let dt = (now - last_display_t).as_secs_f64();
                            let display = (((last_display as f64 + rate * dt) as u64)
                                .min(total)
                                .min(n))
                            .max(last_display);
                            if tty {
                                render_bar(display, n, rate, termw, true);
                            } else if now - last_log >= Duration::from_secs(30)
                                || total >= n
                            {
                                render_bar(display, n, rate, termw, false);
                                last_log = now;
                            }
                            last_display = display;
                            last_display_t = now;
                            if finished.load(Ordering::Relaxed) {
                                if total >= n {
                                    render_bar(n, n, rate, termw, tty);
                                }
                                if tty {
                                    eprintln!();
                                }
                                bar_done.store(true, Ordering::Relaxed);
                                break;
                            }
                        }
                        if !bar_done.load(Ordering::Relaxed) {
                            bar_done.store(true, Ordering::Relaxed);
                        }
                    });
                }
                let rc = unsafe {
                    if let Some(f) = b.sim_seed_ckpt {
                        f(
                            eitem,
                            n,
                            par,
                            shop,
                            results_c.as_ptr() as *const u8,
                            sold_c.as_ptr() as *const u8,
                            ckpt_c.as_ptr() as *const u8,
                            &mut seconds,
                            Some(backend_progress),
                            Arc::as_ptr(&done) as *mut c_void,
                            seed,
                        )
                    } else {
                        (b.sim_ckpt.unwrap())(
                            eitem,
                            n,
                            par,
                            shop,
                            results_c.as_ptr() as *const u8,
                            sold_c.as_ptr() as *const u8,
                            ckpt_c.as_ptr() as *const u8,
                            &mut seconds,
                            Some(backend_progress),
                            Arc::as_ptr(&done) as *mut c_void,
                        )
                    }
                };
                finished.store(true, Ordering::Relaxed);
                let wait_start = Instant::now();
                while !bar_done.load(Ordering::Relaxed)
                    && wait_start.elapsed() < Duration::from_millis(300)
                {
                    std::thread::sleep(Duration::from_millis(5));
                }
                if rc != 0 {
                    eprintln!("{} backend failed (rc={})", b.label, rc);
                    if fs::metadata(&ckpt_file).is_ok() {
                        eprintln!("checkpoint kept at {} (rerun to resume)", ckpt_file);
                    }
                } else {
                    eprintln!("{} sim done in {:.1}s", b.label, seconds);
                    backend_ok = true;
                }
            });
        } else {
            let rc = unsafe {
                if let Some(f) = b.sim_seed {
                    f(
                        eitem,
                        n,
                        par,
                        shop,
                        results_c.as_ptr() as *const u8,
                        sold_c.as_ptr() as *const u8,
                        &mut seconds,
                        seed,
                    )
                } else {
                    (b.sim)(
                        eitem,
                        n,
                        par,
                        shop,
                        results_c.as_ptr() as *const u8,
                        sold_c.as_ptr() as *const u8,
                        &mut seconds,
                    )
                }
            };
            if rc != 0 {
                eprintln!(
                    "{} backend failed (rc={}), trying next backend",
                    b.label, rc
                );
            } else {
                eprintln!("{} sim done in {:.1}s (no progress bar: old backend, rebuild for --backend progress)", b.label, seconds);
                backend_ok = true;
            }
        }
        if backend_ok {
            break;
        }
    }
    if !backend_ok {
        eprintln!(
            "fatal: no usable sim backend (missing {} / {}, or backend failed)",
            CPU_LIB, GPU_LIB
        );
        std::process::exit(1);
    }
    let results_text = fs::read_to_string(&results_file).unwrap_or_default();
    let sold_text = fs::read_to_string(&sold_file).unwrap_or_default();
    if apply {
        if !have_save {
            eprintln!("no save found (results only, skipping merge; copy {} back to merge locally)", results_file);
        } else {
            eprintln!("merging into save (--dry-run to skip)");
            match merge_into_save(&sp, true, &results_text, &sold_text, pack_cost) {
                Ok(()) => eprintln!("merge ok"),
                Err(e) => {
                    eprintln!("merge FAILED: {}", e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        eprintln!("dry run (results files only)");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("{}", usage());
        std::process::exit(2);
    }
    match args[0].as_str() {
        "help" | "--help" | "-h" => println!("{}", usage()),
        "list" => list_packs(),
        "status" => status(),
        "merge" => cmd_merge(&args[1..]),
        "export" => cmd_export(&args[1..]),
        "benchmark" | "bench" => cmd_benchmark(&args[1..]),
        eitem_str => cmd_simulate(eitem_str, &args[1..]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_seed_str() {
        assert_eq!(parse_seed_str("12345"), Some(12345));
        assert_eq!(
            parse_seed_str("0x12345678abcdef01"),
            Some(0x12345678abcdef01)
        );
        assert_eq!(parse_seed_str("0XABCDEF"), Some(0xabcdef));
        assert_eq!(parse_seed_str("invalid"), None);
        assert_eq!(parse_seed_str(""), None);
    }

    #[test]
    fn test_resolve_seed_val() {
        assert_eq!(resolve_seed_val(Some(42)), (42, true));
    }

    #[test]
    fn test_peek_ckpt_seed_v3_and_v2() {
        let dir = std::env::temp_dir();
        let v3_path = dir.join("test_peek_v3.ckpt");
        let v2_path = dir.join("test_peek_v2.ckpt");
        let bad_path = dir.join("test_peek_bad.ckpt");

        let mut v3_data = Vec::new();
        v3_data.extend_from_slice(b"PKCKPT03");
        v3_data.extend_from_slice(&1u32.to_le_bytes()); // tag
        v3_data.extend_from_slice(&129i32.to_le_bytes()); // ei
        v3_data.extend_from_slice(&10i32.to_le_bytes()); // sh
        v3_data.extend_from_slice(&1000u64.to_le_bytes()); // target
        v3_data.extend_from_slice(&50u64.to_le_bytes()); // done_chunks
        v3_data.extend_from_slice(&100u64.to_le_bytes()); // chunks
        v3_data.extend_from_slice(&12u64.to_le_bytes()); // threads
        v3_data.extend_from_slice(&0xcafe_babe_dead_beefu64.to_le_bytes()); // seed
        fs::write(&v3_path, &v3_data).unwrap();

        assert_eq!(
            peek_ckpt_seed(v3_path.to_str().unwrap()),
            Some(0xcafe_babe_dead_beef)
        );
        let _ = fs::remove_file(v3_path);

        let mut v2_data = Vec::new();
        v2_data.extend_from_slice(b"PKCKPT02");
        v2_data.extend_from_slice(&[0u8; 52]);
        fs::write(&v2_path, &v2_data).unwrap();

        assert_eq!(
            peek_ckpt_seed(v2_path.to_str().unwrap()),
            Some(0x12345678abcdef01)
        );
        let _ = fs::remove_file(v2_path);

        fs::write(&bad_path, b"NOT_A_CKPT_FILE").unwrap();
        assert_eq!(peek_ckpt_seed(bad_path.to_str().unwrap()), None);
        let _ = fs::remove_file(bad_path);
    }

    #[test]
    fn test_packmap() {
        assert_eq!(packmap(0), Some((0, 0)));
        assert_eq!(packmap(129), Some((15, 7)));
        assert_eq!(packmap(16), Some((8, 2)));
        assert_eq!(packmap(999999), None);
    }

    #[test]
    fn test_formatting() {
        assert_eq!(human(1500.0), "1.50K");
        assert_eq!(human(2500000.0), "2.50M");
        assert_eq!(money(1.61), "$1.61");
    }
}
