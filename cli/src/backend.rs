use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};

pub fn lib_candidates(name: &str) -> Vec<std::path::PathBuf> {
    let mut names = vec![name.to_string()];

    // Generate cross-platform candidate names (.so, .dylib, .dll)
    let base = name.strip_prefix("lib").unwrap_or(name);
    let stem = base
        .strip_suffix(".so")
        .or_else(|| base.strip_suffix(".dylib"))
        .or_else(|| base.strip_suffix(".dll"))
        .unwrap_or(base);

    for variant in &[
        format!("lib{}.so", stem),
        format!("{}.so", stem),
        format!("lib{}.dylib", stem),
        format!("{}.dylib", stem),
        format!("{}.dll", stem),
        format!("lib{}.dll", stem),
    ] {
        if !names.contains(variant) {
            names.push(variant.clone());
        }
    }

    let mut dirs = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd.clone());
        dirs.push(cwd.join("build"));
        dirs.push(cwd.join("build").join("Release"));
        dirs.push(cwd.join("target").join("release"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            dirs.push(dir.to_path_buf());
            dirs.push(dir.join("build"));
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(std::path::PathBuf::from(format!("{}/.local/lib", home)));
    }
    #[cfg(windows)]
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        dirs.push(std::path::PathBuf::from(format!("{}/.local/lib", userprofile)));
    }
    dirs.push(std::path::PathBuf::from("/usr/local/lib"));
    dirs.push(std::path::PathBuf::from("/usr/lib"));

    let mut v = Vec::new();
    for d in &dirs {
        for n in &names {
            let p = d.join(n);
            if !v.contains(&p) {
                v.push(p);
            }
        }
    }
    // Also include relative names for OS loader search (LD_LIBRARY_PATH, PATH, etc.)
    for n in &names {
        let p = std::path::PathBuf::from(n);
        if !v.contains(&p) {
            v.push(p);
        }
    }
    v
}

pub type ProbeFn = unsafe extern "C" fn() -> i32;
pub type SimFn = unsafe extern "C" fn(i32, u64, i32, i32, *const u8, *const u8, *mut f64) -> i32;
pub type SimSeedFn =
    unsafe extern "C" fn(i32, u64, i32, i32, *const u8, *const u8, *mut f64, u64) -> i32;
pub type SimCkptFn = unsafe extern "C" fn(
    i32,
    u64,
    i32,
    i32,
    *const u8,
    *const u8,
    *const u8,
    *mut f64,
    Option<unsafe extern "C" fn(u64, u64, *mut c_void)>,
    *mut c_void,
) -> i32;
pub type SimSeedCkptFn = unsafe extern "C" fn(
    i32,
    u64,
    i32,
    i32,
    *const u8,
    *const u8,
    *const u8,
    *mut f64,
    Option<unsafe extern "C" fn(u64, u64, *mut c_void)>,
    *mut c_void,
    u64,
) -> i32;

pub struct Backend {
    _lib: libloading::Library,
    pub probe: ProbeFn,
    pub sim: SimFn,
    pub sim_ckpt: Option<SimCkptFn>,
    pub sim_seed: Option<SimSeedFn>,
    pub sim_seed_ckpt: Option<SimSeedCkptFn>,
    pub label: &'static str,
}

unsafe impl Send for Backend {}

pub fn load_backend(name: &'static str, label: &'static str) -> Option<Backend> {
    for path in lib_candidates(name) {
        let lib = match unsafe { libloading::Library::new(&path) } {
            Ok(l) => l,
            Err(_) => continue,
        };
        let probe = match unsafe { lib.get::<ProbeFn>(b"packopen_probe") } {
            Ok(f) => *f,
            Err(_) => continue,
        };
        let sim = match unsafe { lib.get::<SimFn>(b"packopen_sim") } {
            Ok(f) => *f,
            Err(_) => continue,
        };
        let sim_ckpt = unsafe { lib.get::<SimCkptFn>(b"packopen_sim_ckpt") }
            .map(|f| *f)
            .ok();
        let sim_seed = unsafe { lib.get::<SimSeedFn>(b"packopen_sim_seed") }
            .map(|f| *f)
            .ok();
        let sim_seed_ckpt = unsafe { lib.get::<SimSeedCkptFn>(b"packopen_sim_seed_ckpt") }
            .map(|f| *f)
            .ok();
        return Some(Backend {
            _lib: lib,
            probe,
            sim,
            sim_ckpt,
            sim_seed,
            sim_seed_ckpt,
            label,
        });
    }
    None
}

pub extern "C" fn backend_progress(done: u64, _total: u64, ctx: *mut c_void) {
    let a = unsafe { &*(ctx as *const AtomicU64) };
    a.store(done, Ordering::Relaxed);
}

pub fn default_threads() -> usize {
    std::thread::available_parallelism()
        .map(|p| (p.get() * 2).clamp(1, 256))
        .unwrap_or(12)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lib_candidates() {
        let candidates = lib_candidates("test_lib.so");
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|p| p.ends_with("test_lib.so")));
    }

    #[test]
    fn test_default_threads() {
        let th = default_threads();
        assert!((1..=256).contains(&th));
    }
}
