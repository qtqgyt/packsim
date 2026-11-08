use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn get_bin() -> String {
    env!("CARGO_BIN_EXE_packopener-cli").to_string()
}

fn remove_backups_for(save_path: &Path) {
    let Some(parent) = save_path.parent() else {
        return;
    };
    let Some(name) = save_path.file_name().and_then(|s| s.to_str()) else {
        return;
    };
    let prefix = format!("{name}.bak_");
    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().starts_with(&prefix) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
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
        .args([
            "0",
            "100",
            "--dry-run",
            "--backend",
            "cpu",
            "--threads",
            "1",
            "--results",
            results_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Exit status should be 0, output: {}",
        String::from_utf8_lossy(&output.stderr)
    );

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
        .args([
            "0",
            "100",
            "--dry-run",
            "--backend",
            "cpu",
            "--threads",
            "1",
            "--seed",
            "0x12345",
            "--results",
            results_file_1.to_str().unwrap(),
            "--sold",
            sold_file_1.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output_1.status.success(),
        "Exit status should be 0 for run 1"
    );

    let output_2 = Command::new(get_bin())
        .env("LD_LIBRARY_PATH", &new_ld_path)
        .current_dir(&root_dir)
        .args([
            "0",
            "100",
            "--dry-run",
            "--backend",
            "cpu",
            "--threads",
            "1",
            "--seed",
            "0x12345",
            "--results",
            results_file_2.to_str().unwrap(),
            "--sold",
            sold_file_2.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output_2.status.success(),
        "Exit status should be 0 for run 2"
    );

    let contents_1 = fs::read_to_string(&results_file_1).expect("Could not read results file 1");
    let contents_2 = fs::read_to_string(&results_file_2).expect("Could not read results file 2");

    assert_eq!(
        contents_1, contents_2,
        "Results should be deterministic with the same seed"
    );

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

    assert!(
        output.status.success(),
        "Merge should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let new_save = fs::read_to_string(&save_path).unwrap();
    assert!(
        new_save.contains("\"m_CardCollectedList\":[7,")
            || new_save.contains("\"m_CardCollectedList\": [7,"),
        "Save should reflect merged card count"
    );

    let _ = fs::remove_file(&save_path);
    let _ = fs::remove_file(&results_path);
    remove_backups_for(&save_path);
}

#[test]
fn test_merge_missing_results_file() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .args([
            "merge",
            "--results",
            "nonexistent_file_definitely_missing.simple",
        ])
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

    assert!(
        output.status.success(),
        "GPU fallback should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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

    assert!(
        output.status.success(),
        "Merge with trailing slash should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = fs::remove_file(&save_path);
    let _ = fs::remove_file(&results_path);
    remove_backups_for(&save_path);
}

#[test]
fn test_simulate_default_is_dry_run() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let temp_dir = std::env::temp_dir();
    let save_path = temp_dir.join(format!("test_sim_default_save_{}.json", std::process::id()));
    let results_path = temp_dir.join(format!(
        "test_sim_default_results_{}.simple",
        std::process::id()
    ));

    let mock_save = r#"{"m_SaveIndex":0,"m_CoinAmountDouble":1000.0,"m_CoinAmount":1000.0,"m_ShopLevel":1,"m_CardCollectedList":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"m_IsCardCollectedList":[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],"m_GenCardMarketPriceList":[{"generatedMarketPrice":1.0,"pricePercentChangeList":0.0}],"m_GeneratedCostPriceList":[1.0],"m_ItemPricePercentChangeList":[0.0]}"#;
    fs::write(&save_path, mock_save).unwrap();

    let ld_path = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
    let new_ld_path = if ld_path.is_empty() {
        root_dir.to_string_lossy().to_string()
    } else {
        format!("{}:{}", root_dir.to_string_lossy(), ld_path)
    };

    let output = Command::new(get_bin())
        .env("LD_LIBRARY_PATH", &new_ld_path)
        .current_dir(&root_dir)
        .args([
            "0",
            "10",
            "--backend",
            "cpu",
            "--threads",
            "1",
            "--save",
            save_path.to_str().unwrap(),
            "--results",
            results_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Exit status should be 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let current_save = fs::read_to_string(&save_path).unwrap();
    assert_eq!(
        current_save, mock_save,
        "Save should not be modified on default dry-run simulation"
    );

    let _ = fs::remove_file(&save_path);
    let _ = fs::remove_file(&results_path);
    let stem = results_path.to_str().unwrap().trim_end_matches(".simple");
    let _ = fs::remove_file(format!("{}_sold.simple", stem));
}

#[test]
fn test_merge_default_is_dry_run_and_creates_no_backup() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let temp_dir = std::env::temp_dir();
    let save_path = temp_dir.join(format!(
        "test_merge_default_save_{}.json",
        std::process::id()
    ));
    let results_path = temp_dir.join(format!(
        "test_merge_default_results_{}.simple",
        std::process::id()
    ));

    let mock_save = r#"{"m_SaveIndex":0,"m_CoinAmountDouble":1000.0,"m_CoinAmount":1000.0,"m_ShopLevel":1,"m_CardCollectedList":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"m_IsCardCollectedList":[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],"m_GenCardMarketPriceList":[{"generatedMarketPrice":1.0,"pricePercentChangeList":0.0}]}"#;
    fs::write(&save_path, mock_save).unwrap();
    fs::write(&results_path, "0 0 0 0 7\n").unwrap();

    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .args([
            "merge",
            "--save",
            save_path.to_str().unwrap(),
            "--results",
            results_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Merge dry-run should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let current_save = fs::read_to_string(&save_path).unwrap();
    assert_eq!(
        current_save, mock_save,
        "Save should not be modified on default dry-run merge"
    );

    // Check no backup was created
    let mut found_backup = false;
    if let Ok(entries) = fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!(
                "test_merge_default_save_{}.json.bak",
                std::process::id()
            )) {
                found_backup = true;
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    assert!(!found_backup, "Dry run must not create backup files");

    let _ = fs::remove_file(&save_path);
    let _ = fs::remove_file(&results_path);
}

#[test]
fn test_applied_merge_creates_backup_and_preserves_previous() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let temp_dir = std::env::temp_dir();
    let save_path = temp_dir.join(format!(
        "test_bak_preserve_save_{}.json",
        std::process::id()
    ));
    let results_path = temp_dir.join(format!(
        "test_bak_preserve_results_{}.simple",
        std::process::id()
    ));

    let mock_save_1 = r#"{"m_SaveIndex":0,"m_CoinAmountDouble":1000.0,"m_CoinAmount":1000.0,"m_ShopLevel":1,"m_CardCollectedList":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"m_IsCardCollectedList":[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],"m_GenCardMarketPriceList":[{"generatedMarketPrice":1.0,"pricePercentChangeList":0.0}]}"#;
    fs::write(&save_path, mock_save_1).unwrap();
    fs::write(&results_path, "0 0 0 0 5\n").unwrap();

    // First applied merge
    let out1 = Command::new(get_bin())
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

    assert!(
        out1.status.success(),
        "First merge should succeed: {}",
        String::from_utf8_lossy(&out1.stderr)
    );
    let stderr1 = String::from_utf8_lossy(&out1.stderr);
    assert!(
        stderr1.contains("backup:"),
        "First merge must report backup path in output: {}",
        stderr1
    );

    // Extract reported backup path from stderr
    let bak1_path = stderr1
        .lines()
        .find(|l| l.contains("backup:"))
        .and_then(|l| l.split("backup:").nth(1))
        .map(|s| s.trim().to_string())
        .expect("Should report backup path");
    let bak1 = PathBuf::from(&bak1_path);
    assert!(bak1.exists(), "Reported backup file must exist");
    let bak1_content = fs::read_to_string(&bak1).unwrap();
    assert_eq!(
        bak1_content, mock_save_1,
        "First backup must contain initial save state"
    );

    // Second applied merge
    let out2 = Command::new(get_bin())
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

    assert!(
        out2.status.success(),
        "Second merge should succeed: {}",
        String::from_utf8_lossy(&out2.stderr)
    );
    let stderr2 = String::from_utf8_lossy(&out2.stderr);
    assert!(
        stderr2.contains("backup:"),
        "Second merge must report backup path: {}",
        stderr2
    );
    let bak2_path = stderr2
        .lines()
        .find(|l| l.contains("backup:"))
        .and_then(|l| l.split("backup:").nth(1))
        .map(|s| s.trim().to_string())
        .expect("Should report backup path");
    let bak2 = PathBuf::from(&bak2_path);
    assert!(bak2.exists(), "Second backup file must exist");

    // Ensure bak1 is preserved and not overwritten!
    assert_ne!(bak1_path, bak2_path, "Backup paths must be unique");
    assert!(
        bak1.exists(),
        "First backup must still exist after second merge"
    );
    let bak1_after = fs::read_to_string(&bak1).unwrap();
    assert_eq!(
        bak1_after, mock_save_1,
        "First backup must not be overwritten by second merge"
    );

    let _ = fs::remove_file(&save_path);
    let _ = fs::remove_file(&results_path);
    let _ = fs::remove_file(&bak1);
    let _ = fs::remove_file(&bak2);
}

#[test]
fn test_merge_validation_errors() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    // Unknown flag
    let out_unknown = Command::new(get_bin())
        .current_dir(&root_dir)
        .args(["merge", "--invalid-flag"])
        .output()
        .expect("Failed to execute command");
    assert_eq!(
        out_unknown.status.code(),
        Some(2),
        "Unknown flag to merge must exit 2"
    );

    // Missing value for --results
    let out_missing_val = Command::new(get_bin())
        .current_dir(&root_dir)
        .args(["merge", "--results"])
        .output()
        .expect("Failed to execute command");
    assert_eq!(
        out_missing_val.status.code(),
        Some(2),
        "Missing value for --results must exit 2"
    );
}

#[test]
fn test_invalid_pack_id_exits_usage_error() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .args(["999", "10"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(2), "Invalid pack ID must exit 2");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown pack id") || stderr.contains("unknown"),
        "Expected error message: {}",
        stderr
    );
}

#[test]
fn test_status_flags_and_missing_reporting() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let temp_dir = std::env::temp_dir();
    let save_path = temp_dir.join(format!("test_status_save_{}.json", std::process::id()));
    let mock_save = r#"{"m_SaveIndex":3,"m_CoinAmountDouble":42.50,"m_ShopLevel":99}"#;
    fs::write(&save_path, mock_save).unwrap();

    // Test --save flag
    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .args(["status", "--save", save_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("slot 3"));
    assert!(stdout.contains("level 99"));
    assert!(stdout.contains("$42.50"));

    // Test missing save reporting clearly
    let missing_path = temp_dir.join(format!("test_missing_save_{}.json", std::process::id()));
    let out_missing = Command::new(get_bin())
        .current_dir(&root_dir)
        .args(["status", "--save", missing_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");
    let stdout_missing = String::from_utf8_lossy(&out_missing.stdout);
    let stderr_missing = String::from_utf8_lossy(&out_missing.stderr);
    let all_out = format!("{}{}", stdout_missing, stderr_missing);
    assert!(
        all_out.contains("missing")
            || all_out.contains("cannot read")
            || all_out.contains("unreadable"),
        "Expected missing report, got: {}",
        all_out
    );

    // Test invalid flag exits 2
    let out_bad_flag = Command::new(get_bin())
        .current_dir(&root_dir)
        .args(["status", "--bogus"])
        .output()
        .expect("Failed to execute command");
    assert_eq!(
        out_bad_flag.status.code(),
        Some(2),
        "Status unknown flag must exit 2"
    );

    let _ = fs::remove_file(&save_path);
}

#[test]
fn test_pack_catalog_matches_backend_expansions() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .arg("list")
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("129"), "List must contain special ID 129");
    assert!(stdout.contains("Ascension"), "List must contain Ascension");
    assert!(
        stdout
            .lines()
            .any(|line| line.split_whitespace().next() == Some("17") && line.contains("Megabot")),
        "ID 17 is the Megabot pack"
    );
    assert!(
        stdout
            .lines()
            .any(|line| line.split_whitespace().next() == Some("18") && line.contains("FantasyRPG")),
        "ID 18 is the FantasyRPG pack"
    );
    assert!(
        stdout
            .lines()
            .any(|line| line.split_whitespace().next() == Some("19") && line.contains("CatJob")),
        "ID 19 is the CatJob pack"
    );
    assert!(
        !stdout.contains("?"),
        "List should not contain '?' for known pack IDs"
    );
}

#[test]
fn test_applied_merge_rejects_removed_expansion_arrays_without_writing_save() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let temp_dir = std::env::temp_dir();
    let results_path = temp_dir.join(format!("test_removed_array_{}.simple", std::process::id()));
    let save_path = temp_dir.join(format!("test_removed_array_{}.json", std::process::id()));
    let original = r#"{"m_CoinAmount":100,"m_CoinAmountDouble":100,"m_ShopLevel":1}"#;
    fs::write(&results_path, "3 0 0 0 1\n").unwrap();
    fs::write(&save_path, original).unwrap();

    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .args([
            "merge",
            results_path.to_str().unwrap(),
            "--apply",
            "--save",
            save_path.to_str().unwrap(),
            "--sold",
            temp_dir.join("missing_sold.simple").to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("save missing required array m_CardCollectedListMegabot"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&save_path).unwrap(), original);
    let backup_prefix = format!("{}.bak_", save_path.file_name().unwrap().to_string_lossy());
    assert!(
        !fs::read_dir(&temp_dir).unwrap().flatten().any(|entry| entry
            .file_name()
            .to_string_lossy()
            .starts_with(&backup_prefix)),
        "failed merges must not create a save backup"
    );
    remove_backups_for(&save_path);
    let _ = fs::remove_file(results_path);
    let _ = fs::remove_file(save_path);
}

#[test]
fn test_applied_simulation_preflights_removed_expansion_arrays() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let temp_dir = std::env::temp_dir();
    let results_path = temp_dir.join(format!("test_preflight_{}.simple", std::process::id()));
    let save_path = temp_dir.join(format!("test_preflight_{}.json", std::process::id()));
    let _ = fs::remove_file(&results_path);
    let original = r#"{"m_CoinAmount":100,"m_CoinAmountDouble":100,"m_ShopLevel":1}"#;
    fs::write(&save_path, original).unwrap();

    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .args([
            "17",
            "1",
            "--apply",
            "--backend",
            "cpu",
            "--save",
            save_path.to_str().unwrap(),
            "--results",
            results_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("save missing required array m_CardCollectedListMegabot"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !results_path.exists(),
        "preflight must run before simulation"
    );
    assert_eq!(fs::read_to_string(&save_path).unwrap(), original);
    remove_backups_for(&save_path);
    let _ = fs::remove_file(results_path);
    let _ = fs::remove_file(save_path);
}

#[test]
fn test_status_rejects_conflicting_slot_and_save() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let temp_dir = std::env::temp_dir();
    let save_path = temp_dir.join(format!(
        "test_conflicting_target_{}.json",
        std::process::id()
    ));
    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .args([
            "status",
            "--slot",
            "1",
            "--save",
            save_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be combined"));
}

#[test]
fn test_merge_rejects_conflicting_slot_and_save() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let temp_dir = std::env::temp_dir();
    let save_path = temp_dir.join(format!(
        "test_conflicting_merge_target_{}.json",
        std::process::id()
    ));
    let results_path = temp_dir.join(format!(
        "test_conflicting_results_{}.simple",
        std::process::id()
    ));
    fs::write(&results_path, "").unwrap();

    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .args([
            "merge",
            "--slot",
            "1",
            "--save",
            save_path.to_str().unwrap(),
            "--results",
            results_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be combined"));
    let _ = fs::remove_file(&results_path);
}

#[test]
fn test_merge_sold_sidecar_derivation_and_absent_note() {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    let temp_dir = std::env::temp_dir();
    let save_path = temp_dir.join(format!("test_sidecar_save_{}.json", std::process::id()));
    let results_path = temp_dir.join(format!(
        "test_sidecar_results_{}.simple",
        std::process::id()
    ));
    let sold_path = temp_dir.join(format!(
        "test_sidecar_results_{}_sold.simple",
        std::process::id()
    ));

    let mock_save = r#"{"m_SaveIndex":0,"m_CoinAmountDouble":100.0,"m_CoinAmount":100.0,"m_ShopLevel":1,"m_CardCollectedList":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"m_IsCardCollectedList":[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],"m_GenCardMarketPriceList":[{"generatedMarketPrice":10.0,"pricePercentChangeList":0.0}]}"#;
    fs::write(&save_path, mock_save).unwrap();
    fs::write(&results_path, "0 0 0 0 1\n").unwrap();
    // Sold card: 0 0 0 0 2 (2 cards sold at 10.0 = 20.0 cash)
    fs::write(&sold_path, "0 0 0 0 2\n").unwrap();

    // Test automatic sidecar derivation
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

    assert!(
        output.status.success(),
        "Merge should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let new_save = fs::read_to_string(&save_path).unwrap();
    assert!(
        new_save.contains("\"m_CoinAmountDouble\":120")
            || new_save.contains("\"m_CoinAmountDouble\": 120"),
        "Save should have added sold cards revenue (+20.0): {}",
        new_save
    );

    // Test absent sold sidecar behavior
    let _ = fs::remove_file(&sold_path);
    let output2 = Command::new(get_bin())
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

    assert!(
        output2.status.success(),
        "Merge with absent sidecar should succeed"
    );
    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    assert!(
        stderr2.contains("not found")
            || stderr2.contains("skipping")
            || stderr2.contains("continuing"),
        "Should clearly report absent sidecar behavior: {}",
        stderr2
    );

    let _ = fs::remove_file(&save_path);
    let _ = fs::remove_file(&results_path);
    let _ = fs::remove_file(&sold_path);
    remove_backups_for(&save_path);
}

#[test]
fn test_merge_dry_run_reports_that_save_was_not_modified() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let temp_dir = std::env::temp_dir();
    let save_path = temp_dir.join(format!(
        "test_merge_dry_message_{}.json",
        std::process::id()
    ));
    let results_path = temp_dir.join(format!(
        "test_merge_dry_message_{}.simple",
        std::process::id()
    ));
    fs::write(
        &save_path,
        r#"{"m_SaveIndex":0,"m_CoinAmountDouble":100.0,"m_ShopLevel":1}"#,
    )
    .unwrap();
    fs::write(&results_path, "").unwrap();

    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .args([
            "merge",
            "--save",
            save_path.to_str().unwrap(),
            "--results",
            results_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("dry run") && stderr.contains("not modified"),
        "unexpected output: {stderr}"
    );
    assert!(
        !stderr.contains("merge ok"),
        "dry-run must not claim that the save was merged: {stderr}"
    );
    let _ = fs::remove_file(&save_path);
    let _ = fs::remove_file(&results_path);
}

#[test]
fn test_simulate_rejects_missing_results_and_sold_paths() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    for flag in ["--results", "--sold"] {
        let output = Command::new(get_bin())
            .current_dir(&root_dir)
            .args(["0", "1", flag])
            .output()
            .expect("Failed to execute command");
        assert_eq!(
            output.status.code(),
            Some(2),
            "{flag} without a path should be a usage error"
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("needs a file path"));
    }
}

#[test]
fn test_merge_derives_sidecar_from_filename_not_dotted_parent() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let temp_dir =
        std::env::temp_dir().join(format!("packsim.sidecar.test.{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();
    let save_path = temp_dir.join("save.json");
    let results_path = temp_dir.join("results");
    let sold_path = temp_dir.join("results_sold.simple");
    fs::write(&save_path, r#"{"m_SaveIndex":0,"m_CoinAmountDouble":100.0,"m_CoinAmount":100.0,"m_ShopLevel":1,"m_CardCollectedList":[0],"m_IsCardCollectedList":[false],"m_GenCardMarketPriceList":[{"generatedMarketPrice":10.0,"pricePercentChangeList":0.0}]}"#).unwrap();
    fs::write(&results_path, "").unwrap();
    fs::write(&sold_path, "0 0 0 0 2\n").unwrap();

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

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let updated = fs::read_to_string(&save_path).unwrap();
    assert!(
        updated.contains("120"),
        "sold sidecar revenue should be applied: {updated}"
    );
    let _ = fs::remove_file(&save_path);
    let _ = fs::remove_file(&results_path);
    let _ = fs::remove_file(&sold_path);
    remove_backups_for(&save_path);
    let _ = fs::remove_dir(temp_dir);
}

#[test]
fn test_simulate_rejects_conflicting_slot_and_save() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let save_path =
        std::env::temp_dir().join(format!("test_sim_conflict_{}.json", std::process::id()));
    let output = Command::new(get_bin())
        .current_dir(&root_dir)
        .args([
            "0",
            "1",
            "--slot",
            "1",
            "--save",
            save_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be combined"));
}
