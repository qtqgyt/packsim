use std::fs::{self, File};
use std::io::{self, Write};

pub fn exp_name(exp: i32) -> &'static str {
    match exp {
        0 => "Tetramon",
        1 => "Destiny",
        2 => "Ghost",
        3 => "Megabot",
        4 => "FantasyRPG",
        5 => "CatJob",
        7 => "Ascension",
        _ => "Unknown",
    }
}

pub fn border_name(border: usize) -> &'static str {
    match border {
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
    let text =
        fs::read_to_string(input_file).map_err(|e| format!("failed to read input file: {}", e))?;
    let entries = crate::simple::parse(&text, input_file)?;
    if format != "csv" && format != "json" {
        return Err("unknown format. Must be 'csv' or 'json'.".to_string());
    }

    let mut out: Box<dyn Write> = match output_file {
        Some(path) => {
            let f =
                File::create(path).map_err(|e| format!("failed to create output file: {}", e))?;
            Box::new(f)
        }
        None => Box::new(io::stdout()),
    };

    if format == "csv" {
        writeln!(
            out,
            "exp,exp_name,card_id,border,border_name,foil,count,destiny"
        )
        .map_err(|e| e.to_string())?;
    } else if format == "json" {
        writeln!(out, "[").map_err(|e| e.to_string())?;
    } else {
        return Err("unknown format. Must be 'csv' or 'json'.".to_string());
    }

    let mut first_json = true;

    for entry in entries {
        let exp = entry.exp;
        let per = crate::save::per_of(exp).max(1) as usize;
        let card_id = entry.save / per;
        let border = entry.save % per;
        let destiny = entry.dest == 1;
        let foil = entry.foil;
        let count = entry.n;

        let ename = exp_name(exp);
        let bname = border_name(border);

        if format == "csv" {
            writeln!(
                out,
                "{},\"{}\",{},{},\"{}\",{},{},{}",
                exp, ename, card_id, border, bname, foil, count, destiny
            )
            .map_err(|e| e.to_string())?;
        } else if format == "json" {
            if !first_json {
                writeln!(out, ",").map_err(|e| e.to_string())?;
            }
            first_json = false;
            write!(
                out,
                r#"  {{"exp": {}, "exp_name": "{}", "card_id": {}, "border": {}, "border_name": "{}", "foil": {}, "count": {}, "destiny": {}}}"#,
                exp, ename, card_id, border, bname, foil, count, destiny
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
    fn malformed_rows_do_not_replace_export_output() {
        let dir = std::env::temp_dir();
        let input = dir.join(format!("simple_bad_input_{}", std::process::id()));
        let output = dir.join(format!("simple_bad_output_{}", std::process::id()));
        for row in [
            "bad 0 0 0 1",
            "0 bad 0 0 1",
            "0 0 bad 0 1",
            "0 0 2 0 1",
            "0 0 0 2 1",
            "0 0 0 0 bad",
            "0 0 0 0",
            "0 0 0 0 1 extra",
            "0 0 0 0 -1",
        ] {
            for format in ["csv", "json"] {
                fs::write(&input, format!("0 0 0 0 1\n{row}\n")).unwrap();
                fs::write(&output, "original").unwrap();
                let error = export_results(
                    input.to_str().unwrap(),
                    Some(output.to_str().unwrap()),
                    format,
                )
                .unwrap_err();
                assert!(error.contains("line 2"), "{row}: {error}");
                assert_eq!(fs::read_to_string(&output).unwrap(), "original");
            }
        }
        let _ = fs::remove_file(input);
        let _ = fs::remove_file(output);
    }

    #[test]
    fn test_export_csv() {
        let dir = std::env::temp_dir();
        let in_path = dir.join(format!("test_exp_in_{}.tmp", std::process::id()));
        let out_path = dir.join(format!("test_exp_out_{}.tmp", std::process::id()));

        fs::write(&in_path, "0 123 0 1 50\n1 245 1 0 25\n2 10 1 1 10\n").unwrap();
        export_results(
            in_path.to_str().unwrap(),
            Some(out_path.to_str().unwrap()),
            "csv",
        )
        .unwrap();

        let out_str = fs::read_to_string(&out_path).unwrap();
        let lines: Vec<&str> = out_str.lines().collect();
        assert_eq!(
            lines[0],
            "exp,exp_name,card_id,border,border_name,foil,count,destiny"
        );
        assert_eq!(lines[1], "0,\"Tetramon\",10,3,\"Gold\",true,50,false");
        assert_eq!(lines[2], "1,\"Destiny\",20,5,\"Full Art\",false,25,true");
        assert_eq!(lines[3], "2,\"Ghost\",5,0,\"Normal\",true,10,true");

        let _ = fs::remove_file(in_path);
        let _ = fs::remove_file(out_path);
    }

    #[test]
    fn test_export_json() {
        let dir = std::env::temp_dir();
        let in_path = dir.join(format!("test_exp_in_j_{}.tmp", std::process::id()));
        let out_path = dir.join(format!("test_exp_out_j_{}.tmp", std::process::id()));

        fs::write(&in_path, "0 123 0 1 50\n1 245 1 0 25\n2 10 1 1 10\n").unwrap();
        export_results(
            in_path.to_str().unwrap(),
            Some(out_path.to_str().unwrap()),
            "json",
        )
        .unwrap();

        let out_str = fs::read_to_string(&out_path).unwrap();
        assert!(out_str.contains(r#"{"exp": 0, "exp_name": "Tetramon", "card_id": 10, "border": 3, "border_name": "Gold", "foil": true, "count": 50, "destiny": false}"#));
        assert!(out_str.contains(r#"{"exp": 1, "exp_name": "Destiny", "card_id": 20, "border": 5, "border_name": "Full Art", "foil": false, "count": 25, "destiny": true}"#));
        assert!(out_str.contains(r#"{"exp": 2, "exp_name": "Ghost", "card_id": 5, "border": 0, "border_name": "Normal", "foil": true, "count": 10, "destiny": true}"#));

        let _ = fs::remove_file(in_path);
        let _ = fs::remove_file(out_path);
    }
}
