use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn exp_name(exp: i32) -> &'static str {
    match exp {
        0 => "Tetramon",
        1 => "Destiny",
        2 => "Ghost",
        7 => "Ascension",
        _ => "Unknown",
    }
}

pub fn border_name(dest: i32) -> &'static str {
    match dest {
        0 => "Normal",
        1 => "1st Edition",
        2 => "Silver",
        3 => "Gold",
        4 => "EX",
        5 => "Full Art",
        _ => "Unknown",
    }
}

pub fn export_results(
    input_file: &str,
    output_file: Option<&str>,
    format: &str,
) -> Result<(), String> {
    let file = File::open(input_file).map_err(|e| format!("failed to open input file: {}", e))?;
    let reader = BufReader::new(file);

    let mut out: Box<dyn Write> = match output_file {
        Some(path) => {
            let f = File::create(path).map_err(|e| format!("failed to create output file: {}", e))?;
            Box::new(f)
        }
        None => Box::new(io::stdout()),
    };

    if format == "csv" {
        writeln!(out, "exp,exp_name,card_id,border,border_name,foil,count")
            .map_err(|e| e.to_string())?;
    } else if format == "json" {
        writeln!(out, "[").map_err(|e| e.to_string())?;
    } else {
        return Err("unknown format. Must be 'csv' or 'json'.".to_string());
    }

    let mut first_json = true;

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 5 {
            continue;
        }

        let exp: i32 = f[0].parse().unwrap_or(0);
        let card_id: i32 = f[1].parse().unwrap_or(0);
        let dest: i32 = f[2].parse().unwrap_or(0);
        let foil: bool = f[3] == "1";
        let count: i64 = f[4].parse().unwrap_or(0);

        let ename = exp_name(exp);
        let bname = border_name(dest);

        if format == "csv" {
            writeln!(
                out,
                "{},\"{}\",{},{},\"{}\",{},{}",
                exp, ename, card_id, dest, bname, foil, count
            )
            .map_err(|e| e.to_string())?;
        } else if format == "json" {
            if !first_json {
                writeln!(out, ",").map_err(|e| e.to_string())?;
            }
            first_json = false;
            write!(
                out,
                r#"  {{"exp": {}, "exp_name": "{}", "card_id": {}, "border": {}, "border_name": "{}", "foil": {}, "count": {}}}"#,
                exp, ename, card_id, dest, bname, foil, count
            )
            .map_err(|e| e.to_string())?;
        }
    }

    if format == "json" {
        writeln!(out, "\n]").map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_export_csv() {
        let dir = std::env::temp_dir();
        let in_path = dir.join(format!("test_exp_in_{}.tmp", std::process::id()));
        let out_path = dir.join(format!("test_exp_out_{}.tmp", std::process::id()));

        fs::write(&in_path, "0 10 3 1 50\n1 20 5 0 25\n").unwrap();
        export_results(in_path.to_str().unwrap(), Some(out_path.to_str().unwrap()), "csv").unwrap();

        let out_str = fs::read_to_string(&out_path).unwrap();
        let lines: Vec<&str> = out_str.lines().collect();
        assert_eq!(lines[0], "exp,exp_name,card_id,border,border_name,foil,count");
        assert_eq!(lines[1], "0,\"Tetramon\",10,3,\"Gold\",true,50");
        assert_eq!(lines[2], "1,\"Destiny\",20,5,\"Full Art\",false,25");

        let _ = fs::remove_file(in_path);
        let _ = fs::remove_file(out_path);
    }

    #[test]
    fn test_export_json() {
        let dir = std::env::temp_dir();
        let in_path = dir.join(format!("test_exp_in_j_{}.tmp", std::process::id()));
        let out_path = dir.join(format!("test_exp_out_j_{}.tmp", std::process::id()));

        fs::write(&in_path, "0 10 3 1 50\n1 20 5 0 25\n").unwrap();
        export_results(in_path.to_str().unwrap(), Some(out_path.to_str().unwrap()), "json").unwrap();

        let out_str = fs::read_to_string(&out_path).unwrap();
        assert!(out_str.contains(r#"{"exp": 0, "exp_name": "Tetramon", "card_id": 10, "border": 3, "border_name": "Gold", "foil": true, "count": 50}"#));
        assert!(out_str.contains(r#"{"exp": 1, "exp_name": "Destiny", "card_id": 20, "border": 5, "border_name": "Full Art", "foil": false, "count": 25}"#));

        let _ = fs::remove_file(in_path);
        let _ = fs::remove_file(out_path);
    }
}
