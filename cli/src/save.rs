use crate::ui::{human, money};
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

pub const INTMAX: i64 = 2147483647;

use crate::json::{
    as_arr, as_arr_mut, as_f64, as_i64, emit, jget, jget_mut, set_bool, set_float, set_int, Parser,
    J,
};

#[cfg(windows)]
pub fn save_dir() -> String {
    let home = std::env::var("USERPROFILE").unwrap_or_default();
    format!("{}/AppData/LocalLow/OPNeonGames/Card Shop Simulator", home)
}

#[cfg(not(windows))]
pub fn save_dir() -> String {
    std::env::var("PACKSIM_SAVE_DIR").unwrap_or_else(|_| {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        format!("{}/.local/share/Steam/steamapps/compatdata/3070070/pfx/drive_c/users/steamuser/AppData/LocalLow/OPNeonGames/Card Shop Simulator", home)
    })
}

pub fn save_path(slot: Option<usize>) -> String {
    let dir = save_dir();
    let dir = dir.trim_end_matches(['/', '\\']);
    match slot {
        Some(n) => format!("{}/savedGames_Release{}.json", dir, n),
        None => format!("{}/savedGames_Release0.json", dir),
    }
}

pub fn json_i64(txt: &str, key: &str) -> Result<i64, ()> {
    let start = txt.find(key).ok_or(())?;
    let rest = &txt[start + key.len()..];
    let rest = rest.trim_start();
    let rest = rest.strip_prefix(':').ok_or(())?;
    let rest = rest.trim_start();
    let len = rest
        .bytes()
        .take_while(|&c| c.is_ascii_digit() || c == b'-')
        .count();
    rest[..len].parse().map_err(|_| ())
}

pub fn json_f64(txt: &str, key: &str) -> Result<f64, ()> {
    let start = txt.find(key).ok_or(())?;
    let rest = &txt[start + key.len()..]
        .trim_start()
        .strip_prefix(':')
        .ok_or(())?;
    let rest = rest.trim_start();
    let len = rest
        .bytes()
        .take_while(|&c| {
            c.is_ascii_digit() || c == b'-' || c == b'.' || c == b'e' || c == b'E' || c == b'+'
        })
        .count();
    rest[..len].parse().map_err(|_| ())
}

pub fn unit_price(save_txt: &str, eitem: i32) -> Option<f64> {
    let base = array_elem(save_txt, "m_GeneratedCostPriceList", eitem)?;
    let pct = array_elem(save_txt, "m_ItemPricePercentChangeList", eitem)?;
    let v = (base + base * (pct / 100.0)) * 100.0;
    let rounded = round_ties_even_f64(v);
    Some(rounded / 100.0)
}

pub fn round_ties_even_f64(x: f64) -> f64 {
    let r = x.floor();
    let diff = x - r;
    if diff > 0.5 {
        r + 1.0
    } else if diff < 0.5 || (r as i64) % 2 == 0 {
        r
    } else {
        r + 1.0
    }
}

pub fn array_elem(txt: &str, key: &str, n: i32) -> Option<f64> {
    let start = txt.find(key)?;
    let mut depth = 0i32;
    let bytes = txt.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            break;
        }
        i += 1;
    }
    i += 1;
    let mut idx = 0i32;
    let mut num = String::new();
    let mut in_num = false;
    while i < bytes.len() && idx <= n {
        let c = bytes[i];
        match c {
            b'[' => {
                depth += 1;
                in_num = false;
                num.clear();
            }
            b']' => {
                if depth == 0 {
                    if idx == n && in_num {
                        return num.parse().ok();
                    }
                    return None;
                }
                depth -= 1;
                in_num = false;
                num.clear();
            }
            b',' if depth == 0 => {
                if idx == n && in_num {
                    return num.parse().ok();
                }
                idx += 1;
                in_num = false;
                num.clear();
            }
            b'{' | b'"' => {
                let open = c;
                let close = if open == b'{' { b'}' } else { b'"' };
                let mut d = 1i32;
                i += 1;
                while i < bytes.len() && d > 0 {
                    if bytes[i] == open && open == b'"' {
                        if bytes[i - 1] != b'\\' {
                            d = 0;
                        }
                    } else if bytes[i] == close && open == b'{' {
                        d -= 1;
                    }
                    i += 1;
                }
                continue;
            }
            b'0'..=b'9' | b'-' | b'.' | b'e' | b'E' | b'+' if depth == 0 => {
                in_num = true;
                num.push(c as char);
            }
            _ => {}
        }
        i += 1;
    }
    None
}

pub fn per_of(exp: i32) -> i32 {
    match exp {
        0 | 1 | 7 => 12,
        2 => 2,
        _ => 12,
    }
}
pub fn per_nofill(exp: i32) -> i32 {
    match exp {
        2 => 1,
        _ => 6,
    }
}

pub fn amt_key(exp: i32, dest: i32) -> Option<&'static str> {
    match (exp, dest) {
        (0, _) => Some("m_CardCollectedList"),
        (1, _) => Some("m_CardCollectedListDestiny"),
        (7, _) => Some("m_CardCollectedListAscension"),
        (2, 0) => Some("m_CardCollectedListGhost"),
        (2, _) => Some("m_CardCollectedListGhostBlack"),
        _ => None,
    }
}
pub fn flag_key(exp: i32, dest: i32) -> Option<&'static str> {
    match (exp, dest) {
        (0, _) => Some("m_IsCardCollectedList"),
        (1, _) => Some("m_IsCardCollectedListDestiny"),
        (7, _) => Some("m_IsCardCollectedListAscension"),
        (2, 0) => Some("m_IsCardCollectedListGhost"),
        (2, _) => Some("m_IsCardCollectedListGhostBlack"),
        _ => None,
    }
}
pub fn gen_price_key(exp: i32, dest: i32) -> Option<&'static str> {
    match (exp, dest) {
        (0, _) => Some("m_GenCardMarketPriceList"),
        (1, _) => Some("m_GenCardMarketPriceListDestiny"),
        (2, 1) => Some("m_GenCardMarketPriceListGhost"),
        (2, _) => Some("m_GenCardMarketPriceListGhostBlack"),
        (7, _) => Some("m_GenCardMarketPriceListAscension"),
        _ => None,
    }
}

pub fn market_value(root: &J, exp: i32, save: usize, dest: i32, foil: bool) -> f64 {
    let gk = match gen_price_key(exp, dest) {
        Some(k) => k,
        None => return 0.0,
    };
    let gen = match jget(root, gk).and_then(as_arr) {
        Some(a) => a,
        None => return 0.0,
    };
    let idx = if foil {
        save + per_nofill(exp) as usize
    } else {
        save
    };
    if idx >= gen.len() {
        return 0.0;
    }
    let e = match &gen[idx] {
        J::Obj(items) => items,
        _ => return 0.0,
    };
    let get = |k: &str| -> f64 {
        e.iter()
            .find(|(kk, _)| kk == k)
            .and_then(|(_, v)| as_f64(v))
            .unwrap_or(0.0)
    };
    let gp = get("generatedMarketPrice") as f32;
    let pp = get("pricePercentChangeList") as f32;
    let p = (gp * (1.0 + pp / 100.0)) * 100.0;
    (p.round_ties_even() as f64) / 100.0
}

pub fn req_level(level: i64) -> i64 {
    250 + level * 200 + level * level * 10
}

struct CardEntry {
    exp: i32,
    save: usize,
    dest: i32,
    foil: bool,
    n: i64,
}

fn parse_card_entry(line: &str, err_msg: &str) -> Result<Option<CardEntry>, String> {
    let mut parts = line.split_whitespace();
    let p0 = match parts.next() {
        Some(s) => s,
        None => return Ok(None),
    };
    let p1 = match parts.next() {
        Some(s) => s,
        None => return Ok(None),
    };
    let p2 = match parts.next() {
        Some(s) => s,
        None => return Ok(None),
    };
    let p3 = match parts.next() {
        Some(s) => s,
        None => return Ok(None),
    };
    let p4 = match parts.next() {
        Some(s) => s,
        None => return Ok(None),
    };

    let exp: i32 = p0.parse().map_err(|_| err_msg.to_string())?;
    let save: usize = p1.parse().map_err(|_| err_msg.to_string())?;
    let dest: i32 = p2.parse().map_err(|_| err_msg.to_string())?;
    let foil: bool = p3 == "1";
    let n: i64 = p4.parse().map_err(|_| err_msg.to_string())?;
    Ok(Some(CardEntry { exp, save, dest, foil, n }))
}

pub fn merge_into_save(
    save_file: &str,
    apply: bool,
    results_text: &str,
    sold_text: &str,
    pack_cost: f64,
) -> Result<(), String> {
    let clean_save = save_file.trim_end_matches(['/', '\\']);
    let clean_save_file = if clean_save.is_empty() {
        save_file
    } else {
        clean_save
    };
    let save_p = Path::new(clean_save_file);
    let file_name = save_p
        .file_name()
        .and_then(|f| f.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("save.json");

    let bak_dir: PathBuf = match std::env::var("PACKSIM_BACKUP_DIR") {
        Ok(dir) if !dir.trim().is_empty() => {
            PathBuf::from(dir.trim_end_matches(['/', '\\']))
        }
        _ => match save_p.parent() {
            Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
            _ => PathBuf::from("."),
        },
    };
    let _ = fs::create_dir_all(&bak_dir);
    let bak = bak_dir.join(format!("{}.bak6", file_name));
    let bak_tmp = bak_dir.join(format!("{}.bak6.tmp{}", file_name, std::process::id()));
    fs::copy(clean_save_file, &bak_tmp).map_err(|e| format!("backup failed: {}", e))?;
    fs::rename(&bak_tmp, &bak).map_err(|e| format!("backup rename failed: {}", e))?;
    let txt = fs::read_to_string(clean_save_file).map_err(|e| format!("read failed: {}", e))?;
    let mut p = Parser {
        b: txt.as_bytes(),
        i: 0,
    };
    let mut root = p
        .parse_value()
        .map_err(|e| format!("parse failed: {}", e))?;

    let mut total_cards: u64 = 0;
    let mut total_xp: i64 = 0;
    let mut total_cash = 0.0f64;
    let mut capped_cards: u64 = 0;
    let mut capped_cash = 0.0f64;
    for line in results_text.lines() {
        let entry = match parse_card_entry(line, "bad results line")? {
            Some(e) => e,
            None => continue,
        };
        total_cards += entry.n as u64;
        let ak = amt_key(entry.exp, entry.dest).ok_or_else(|| format!("bad exp {}", entry.exp))?;
        let fk = flag_key(entry.exp, entry.dest).ok_or_else(|| format!("bad exp {}", entry.exp))?;
        let adj_save = if entry.foil {
            entry.save + per_nofill(entry.exp) as usize
        } else {
            entry.save
        };
        let unit_value = market_value(&root, entry.exp, entry.save, entry.dest, entry.foil);
        {
            let arr = jget_mut(&mut root, ak)
                .and_then(as_arr_mut)
                .ok_or_else(|| format!("missing {}", ak))?;
            if adj_save >= arr.len() {
                return Err(format!("out of range {}/{}", entry.exp, adj_save));
            }
            let cur = as_i64(&arr[adj_save]).unwrap_or(0);
            let new_total = cur + entry.n;
            if new_total > INTMAX {
                let excess = new_total - INTMAX;
                set_int(&mut arr[adj_save], INTMAX);
                let cash = unit_value * (excess as f64);
                total_cash += cash;
                capped_cards += excess as u64;
                capped_cash += cash;
            } else {
                set_int(&mut arr[adj_save], new_total);
            }
        }
        if entry.n > 0 {
            let flg = jget_mut(&mut root, fk)
                .and_then(as_arr_mut)
                .ok_or_else(|| format!("missing {}", fk))?;
            if adj_save < flg.len() {
                set_bool(&mut flg[adj_save], true);
            }
        }
        let b = (entry.save % per_of(entry.exp) as usize) as i64;
        let mut num = (b + 1) * ((b + 2) / 2);
        if entry.foil {
            num *= 8;
        }
        total_xp += (num / 2) * entry.n;
    }

    let mut sold_lines: u64 = 0;
    for line in sold_text.lines() {
        let entry = match parse_card_entry(line, "bad sold line")? {
            Some(e) => e,
            None => continue,
        };
        sold_lines += entry.n as u64;
        total_cash += market_value(&root, entry.exp, entry.save, entry.dest, entry.foil) * (entry.n as f64);
    }
    let mut sweep_cards: u64 = 0;
    let mut sweep_cash = 0.0f64;
    for (ak, exp, dest) in [
        ("m_CardCollectedList", 0, 0),
        ("m_CardCollectedListDestiny", 1, 0),
        ("m_CardCollectedListGhost", 2, 0),
        ("m_CardCollectedListGhostBlack", 2, 1),
        ("m_CardCollectedListAscension", 7, 0),
    ] {
        let per = per_of(exp) as usize;
        let nf = per_nofill(exp) as usize;
        let over: Vec<(usize, i64)> = match jget(&root, ak).and_then(as_arr) {
            Some(a) => a
                .iter()
                .enumerate()
                .filter_map(|(i, v)| {
                    let c = as_i64(v).unwrap_or(0);
                    if c > INTMAX {
                        Some((i, c))
                    } else {
                        None
                    }
                })
                .collect(),
            None => continue,
        };
        for (idx, cur) in over {
            let is_foil = (idx % per) >= nf;
            let base = if is_foil { idx - nf } else { idx };
            let price = market_value(&root, exp, base, dest, is_foil);
            let excess = cur - INTMAX;
            if let Some(arr) = jget_mut(&mut root, ak).and_then(as_arr_mut) {
                set_int(&mut arr[idx], INTMAX);
            }
            let cash = price * (excess as f64);
            total_cash += cash;
            sweep_cards += excess as u64;
            sweep_cash += cash;
        }
    }
    eprintln!(
        "cards: {}, xp: {}, sold: {} -> {}{}{}",
        human(total_cards as f64),
        human(total_xp as f64),
        human(sold_lines as f64),
        money(total_cash),
        if capped_cards > 0 {
            format!(
                " (cap-sold {} cards -> {})",
                human(capped_cards as f64),
                money(capped_cash)
            )
        } else {
            String::new()
        },
        if sweep_cards > 0 {
            format!(
                " (sweep-sold {} old cards -> {})",
                human(sweep_cards as f64),
                money(sweep_cash)
            )
        } else {
            String::new()
        }
    );

    for rep in [
        "m_GameReportDataCollect",
        "m_GameReportDataCollectPermanent",
    ] {
        if let Some(r) = jget_mut(&mut root, rep) {
            if let Some(v) = jget_mut(r, "cardPackOpened") {
                let cur = as_i64(v).unwrap_or(0);
                let newv = cur + (total_cards / 7) as i64;
                let capped = newv.min(INTMAX);
                set_int(v, capped);
                eprintln!(
                    "{} packed opened -> {} {}",
                    rep,
                    human(capped as f64),
                    if newv > INTMAX { "(clamped)" } else { "" }
                );
            }
        }
    }

    let level = jget(&root, "m_ShopLevel").and_then(as_i64).unwrap_or(0);
    let mut exp_point = jget(&root, "m_ShopExpPoint").and_then(as_i64).unwrap_or(0);
    eprintln!("current: level {}, exp {}", level, human(exp_point as f64));
    let mut lvl = level;
    exp_point += total_xp;
    while exp_point >= req_level(lvl) {
        exp_point -= req_level(lvl);
        lvl += 1;
    }
    if let Some(v) = jget_mut(&mut root, "m_ShopLevel") {
        set_int(v, lvl);
    }
    if let Some(v) = jget_mut(&mut root, "m_ShopExpPoint") {
        set_int(v, exp_point);
    }
    eprintln!(
        "after apply: level {}, exp {} (next needs {})",
        lvl,
        human(exp_point as f64),
        human(req_level(lvl) as f64)
    );
    if let Some(v) = jget_mut(&mut root, "m_CoinAmountDouble") {
        let cur = as_f64(v).unwrap_or(0.0);
        let new_coins = cur - pack_cost + total_cash;
        set_float(v, new_coins);
        let tty = std::io::stderr().is_terminal();
        let (red, grn, dim, rst) = if tty {
            ("\x1b[31m", "\x1b[32m", "\x1b[2m", "\x1b[0m")
        } else {
            ("", "", "", "")
        };
        let signed = |v: f64, color: &str| -> String {
            if v >= 0.0 {
                format!("{color}+{}{rst}", money(v))
            } else {
                format!("{color}-{}{rst}", money(v))
            }
        };
        let neg = if new_coins < 0.0 {
            format!(" {red}(NEGATIVE: count exceeded affordable){rst}")
        } else {
            String::new()
        };
        eprintln!(
            "{dim}coins:      {rst}{}{rst}\n\
             {dim}pack cost: {rst}{}\n\
             {dim}sold:      {rst}{}\n\
             {dim}net:       {rst}{}{neg}",
            money(cur),
            signed(-pack_cost, red),
            signed(total_cash, grn),
            signed(new_coins - cur, if new_coins >= cur { grn } else { red })
        );
    }
    if let Some(v) = jget_mut(&mut root, "m_CoinAmount") {
        let cur = as_f64(v).unwrap_or(0.0);
        set_float(v, (cur - pack_cost + total_cash) as f32 as f64);
    }

    if apply {
        let mut out = String::with_capacity(txt.len() + 1024);
        emit(&root, &mut out);
        let save_tmp = format!("{}.tmp{}", clean_save_file, std::process::id());
        fs::write(&save_tmp, out).map_err(|e| format!("write failed: {}", e))?;
        fs::rename(&save_tmp, clean_save_file).map_err(|e| format!("rename failed: {}", e))?;
        eprintln!("Done");
    } else {
        eprintln!("dry run (rerun with --apply to write)");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_i64_and_f64() {
        let txt = r#"{"m_SaveIndex": 3, "m_CoinAmountDouble": 12345.67, "m_ShopLevel": 42}"#;
        assert_eq!(json_i64(txt, "\"m_SaveIndex\""), Ok(3));
        assert_eq!(json_i64(txt, "\"m_ShopLevel\""), Ok(42));
        assert!((json_f64(txt, "\"m_CoinAmountDouble\"").unwrap() - 12345.67).abs() < 1e-4);
    }

    #[test]
    fn test_save_path() {
        let p0 = save_path(Some(0));
        assert!(p0.ends_with("savedGames_Release0.json"));
        let p1 = save_path(Some(1));
        assert!(p1.ends_with("savedGames_Release1.json"));
    }

    #[test]
    fn test_merge_into_save_errors() {
        let res = merge_into_save(
            "nonexistent_file_that_should_not_exist.json",
            false,
            "",
            "",
            0.0,
        );
        assert!(res.is_err());
        assert!(res.unwrap_err().starts_with("backup failed:"));

        let temp_dir = std::env::temp_dir();
        let bad_json_path = temp_dir.join(format!("bad_json_{}.json", std::process::id()));
        std::fs::write(&bad_json_path, "{ bad json").unwrap();

        let res = merge_into_save(bad_json_path.to_str().unwrap(), false, "", "", 0.0);
        assert!(res.is_err());
        assert!(res.unwrap_err().starts_with("parse failed:"));

        // Path with trailing slash does not panic or create empty backup name
        let path_with_slash = format!("{}/", bad_json_path.to_str().unwrap());
        let res_slash = merge_into_save(&path_with_slash, false, "", "", 0.0);
        assert!(res_slash.is_err());
        assert!(res_slash.unwrap_err().starts_with("parse failed:"));

        let _ = std::fs::remove_file(&bad_json_path);
        let bak_path = temp_dir.join(format!("bad_json_{}.json.bak6", std::process::id()));
        let _ = std::fs::remove_file(&bak_path);
    }

    #[test]
    fn test_save_path_trailing_slash() {
        let orig = std::env::var("PACKSIM_SAVE_DIR").ok();
        std::env::set_var("PACKSIM_SAVE_DIR", "/custom/saves/");
        let p = save_path(Some(2));
        assert_eq!(p, "/custom/saves/savedGames_Release2.json");
        if let Some(o) = orig {
            std::env::set_var("PACKSIM_SAVE_DIR", o);
        } else {
            std::env::remove_var("PACKSIM_SAVE_DIR");
        }
    }
}
