#[derive(Debug, PartialEq, Eq)]
pub struct CardEntry {
    pub exp: i32,
    pub save: usize,
    pub dest: i32,
    pub foil: bool,
    pub n: i64,
}

pub fn parse(text: &str, source: &str) -> Result<Vec<CardEntry>, String> {
    let mut entries = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.is_empty() {
            continue;
        }
        let error = |detail: &str| format!("{} line {}: {}: {:?}", source, index + 1, detail, line);
        if fields.len() != 5 {
            return Err(error("expected five fields"));
        }
        let exp = fields[0].parse().map_err(|_| error("invalid expansion"))?;
        if !(0..=15).contains(&exp) {
            return Err(error("expansion must be 0..=15"));
        }
        let subvariant = fields[1]
            .parse::<i64>()
            .map_err(|_| error("invalid card ID"))?;
        if !(0..=4095).contains(&subvariant) {
            return Err(error("subvariant must be 0..=4095"));
        }
        let save = subvariant as usize;
        let dest = match fields[2] {
            "0" => 0,
            "1" => 1,
            _ => return Err(error("dest must be 0 or 1")),
        };
        let foil = match fields[3] {
            "0" => false,
            "1" => true,
            _ => return Err(error("foil must be 0 or 1")),
        };
        let n = fields[4]
            .parse::<i64>()
            .map_err(|_| error("invalid count"))?;
        if n < 0 {
            return Err(error("count must be nonnegative"));
        }
        entries.push(CardEntry {
            exp,
            save,
            dest,
            foil,
            n,
        });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dest_validation() {
        assert!(parse("0 0 0 0 1", "test").is_ok());
        assert!(parse("0 0 1 0 1", "test").is_ok());
        let err = parse("0 0 2 0 1", "test").unwrap_err();
        assert!(err.contains("dest must be 0 or 1"), "{err}");
    }

    #[test]
    fn test_parse_native_field_ranges() {
        assert!(parse("0 0 0 0 1", "test").is_ok());
        assert!(parse("15 4095 0 0 1", "test").is_ok());

        for row in ["-1 0 0 0 1", "16 0 0 0 1"] {
            let err = parse(row, "test").unwrap_err();
            assert!(err.contains("expansion must be 0..=15"), "{err}");
        }
        for row in ["0 -1 0 0 1", "0 4096 0 0 1"] {
            let err = parse(row, "test").unwrap_err();
            assert!(err.contains("subvariant must be 0..=4095"), "{err}");
        }
    }
}
