use std::process::Command;
use std::fs;
use std::path::PathBuf;

fn get_bin() -> String {
    env!("CARGO_BIN_EXE_packopener-cli").to_string()
}

#[test]
fn test_help() {
    let output = Command::new(get_bin())
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Exit status should be 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
}

#[test]
fn test_invalid_arguments() {
    let output = Command::new(get_bin())
        .arg("invalid_argument")
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(2), "Exit status should be 2");
}

#[test]
fn test_dry_run_simulation() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let results_file = root_dir.join("test_results.simple");
    let sold_file = root_dir.join("test_results_sold.simple");

    let ld_path = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
    let new_ld_path = if ld_path.is_empty() {
        root_dir.to_string_lossy().to_string()
    } else {
        format!("{}:{}", root_dir.to_string_lossy(), ld_path)
    };

    let output = Command::new(get_bin())
        .env("LD_LIBRARY_PATH", &new_ld_path)
        .current_dir(&root_dir)
        .args(["0", "100", "--dry-run", "--backend", "cpu", "--threads", "1", "--results", results_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Exit status should be 0, output: {}", String::from_utf8_lossy(&output.stderr));

    let metadata = fs::metadata(&results_file).expect("Results file should exist");
    assert!(metadata.len() > 0, "Results file should not be empty");

    fs::remove_file(&results_file).unwrap_or(());
    fs::remove_file(&sold_file).unwrap_or(());
}

#[test]
fn test_deterministic_seed() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let results_file_1 = root_dir.join("test_results_seed_1.simple");
    let sold_file_1 = root_dir.join("test_results_seed_sold_1.simple");
    let results_file_2 = root_dir.join("test_results_seed_2.simple");
    let sold_file_2 = root_dir.join("test_results_seed_sold_2.simple");

    let ld_path = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
    let new_ld_path = if ld_path.is_empty() {
        root_dir.to_string_lossy().to_string()
    } else {
        format!("{}:{}", root_dir.to_string_lossy(), ld_path)
    };

    let output_1 = Command::new(get_bin())
        .env("LD_LIBRARY_PATH", &new_ld_path)
        .current_dir(&root_dir)
        .args(["0", "100", "--dry-run", "--backend", "cpu", "--threads", "1", "--seed", "0x12345", "--results", results_file_1.to_str().unwrap(), "--sold", sold_file_1.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output_1.status.success(), "Exit status should be 0 for run 1");

    let output_2 = Command::new(get_bin())
        .env("LD_LIBRARY_PATH", &new_ld_path)
        .current_dir(&root_dir)
        .args(["0", "100", "--dry-run", "--backend", "cpu", "--threads", "1", "--seed", "0x12345", "--results", results_file_2.to_str().unwrap(), "--sold", sold_file_2.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output_2.status.success(), "Exit status should be 0 for run 2");

    let contents_1 = fs::read_to_string(&results_file_1).expect("Could not read results file 1");
    let contents_2 = fs::read_to_string(&results_file_2).expect("Could not read results file 2");

    assert_eq!(contents_1, contents_2, "Results should be deterministic with the same seed");

    fs::remove_file(&results_file_1).unwrap_or(());
    fs::remove_file(&sold_file_1).unwrap_or(());
    fs::remove_file(&results_file_2).unwrap_or(());
    fs::remove_file(&sold_file_2).unwrap_or(());
}

#[test]
fn test_merge_command_reads_results() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let temp_dir = std::env::temp_dir();
    let save_path = temp_dir.join(format!("test_merge_save_{}.json", std::process::id()));
    let results_path = temp_dir.join(format!("test_merge_results_{}.simple", std::process::id()));

    let mock_save = r#"{"m_SaveIndex":0,"m_CoinAmountDouble":1000.0,"m_CoinAmount":1000.0,"m_ShopLevel":1,"m_CardCollectedList":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"m_IsCardCollectedList":[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],"m_GenCardMarketPriceList":[{"generatedMarketPrice":1.0,"pricePercentChangeList":0.0}]}"#;
    fs::write(&save_path, mock_save).unwrap();
    fs::write(&results_path, "0 0 0 0 7\n").unwrap();

    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .args([
            "merge",
            "--apply",
            "--save",
            save_path.to_str().unwrap(),
            "--results",
            results_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Merge should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let new_save = fs::read_to_string(&save_path).unwrap();
    assert!(new_save.contains("\"m_CardCollectedList\":[7,") || new_save.contains("\"m_CardCollectedList\": [7,"), "Save should reflect merged card count");

    let _ = fs::remove_file(&save_path);
    let _ = fs::remove_file(&results_path);
    let bak_path = temp_dir.join(format!("test_merge_save_{}.json.bak6", std::process::id()));
    let _ = fs::remove_file(&bak_path);
}

#[test]
fn test_merge_missing_results_file() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .args(["merge", "--results", "nonexistent_file_definitely_missing.simple"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot read nonexistent_file_definitely_missing.simple"));
}

#[test]
fn test_gpu_to_cpu_fallback() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let ld_path = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
    let new_ld_path = if ld_path.is_empty() {
        root_dir.to_string_lossy().to_string()
    } else {
        format!("{}:{}", root_dir.to_string_lossy(), ld_path)
    };

    let output = Command::new(get_bin())
        .env("LD_LIBRARY_PATH", &new_ld_path)
        .env("CUDA_VISIBLE_DEVICES", "")
        .current_dir(&root_dir)
        .args(["0", "10", "--dry-run", "--backend", "gpu", "--threads", "1"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "GPU fallback should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("falling back to CPU"));
}

#[test]
fn test_save_path_trailing_slash_cli() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let temp_dir = std::env::temp_dir();
    let save_path = temp_dir.join(format!("test_slash_save_{}.json", std::process::id()));
    let results_path = temp_dir.join(format!("test_slash_results_{}.simple", std::process::id()));

    let mock_save = r#"{"m_SaveIndex":0,"m_CoinAmountDouble":1000.0,"m_CoinAmount":1000.0,"m_ShopLevel":1,"m_CardCollectedList":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"m_IsCardCollectedList":[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],"m_GenCardMarketPriceList":[{"generatedMarketPrice":1.0,"pricePercentChangeList":0.0}]}"#;
    fs::write(&save_path, mock_save).unwrap();
    fs::write(&results_path, "0 0 0 0 3\n").unwrap();

    let save_arg_with_slash = format!("{}/", save_path.to_str().unwrap());
    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .args([
            "merge",
            "--apply",
            "--save",
            &save_arg_with_slash,
            "--results",
            results_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Merge with trailing slash should succeed: {}", String::from_utf8_lossy(&output.stderr));

    let _ = fs::remove_file(&save_path);
    let _ = fs::remove_file(&results_path);
    let bak_path = temp_dir.join(format!("test_slash_save_{}.json.bak6", std::process::id()));
    let _ = fs::remove_file(&bak_path);
}
