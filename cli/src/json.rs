pub enum J {
    Null,
    Bool(bool),
    Num {
        raw: String,
        ival: Option<i64>,
        fval: f64,
    },
    Str {
        raw: String,
        val: String,
    },
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

pub struct Parser<'a> {
    pub b: &'a [u8],
    pub i: usize,
}

impl<'a> Parser<'a> {
    pub fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }
    pub fn parse_value(&mut self) -> Result<J, String> {
        self.ws();
        if self.i >= self.b.len() {
            return Err("eof".into());
        }
        match self.b[self.i] {
            b'{' => self.parse_obj(),
            b'[' => self.parse_arr(),
            b'"' => self.parse_str(),
            b't' => self.lit("true", J::Bool(true)),
            b'f' => self.lit("false", J::Bool(false)),
            b'n' => self.lit("null", J::Null),
            _ => self.parse_num(),
        }
    }
    pub fn lit(&mut self, s: &str, v: J) -> Result<J, String> {
        if self.b[self.i..].starts_with(s.as_bytes()) {
            self.i += s.len();
            Ok(v)
        } else {
            Err(format!("bad literal at {}", self.i))
        }
    }
    pub fn parse_obj(&mut self) -> Result<J, String> {
        self.i += 1;
        let mut items = Vec::new();
        self.ws();
        if self.i < self.b.len() && self.b[self.i] == b'}' {
            self.i += 1;
            return Ok(J::Obj(items));
        }
        loop {
            self.ws();
            let key = match self.parse_str()? {
                J::Str { val, .. } => val,
                _ => unreachable!(),
            };
            self.ws();
            if self.i >= self.b.len() || self.b[self.i] != b':' {
                return Err("expected :".into());
            }
            self.i += 1;
            let val = self.parse_value()?;
            items.push((key, val));
            self.ws();
            if self.i >= self.b.len() {
                return Err("eof in obj".into());
            }
            match self.b[self.i] {
                b',' => {
                    self.i += 1;
                }
                b'}' => {
                    self.i += 1;
                    break;
                }
                _ => return Err("expected , or }".into()),
            }
        }
        Ok(J::Obj(items))
    }
    pub fn parse_arr(&mut self) -> Result<J, String> {
        self.i += 1;
        let mut items = Vec::new();
        self.ws();
        if self.i < self.b.len() && self.b[self.i] == b']' {
            self.i += 1;
            return Ok(J::Arr(items));
        }
        loop {
            let val = self.parse_value()?;
            items.push(val);
            self.ws();
            if self.i >= self.b.len() {
                return Err("eof in arr".into());
            }
            match self.b[self.i] {
                b',' => {
                    self.i += 1;
                }
                b']' => {
                    self.i += 1;
                    break;
                }
                _ => return Err("expected , or ]".into()),
            }
        }
        Ok(J::Arr(items))
    }
    pub fn parse_str(&mut self) -> Result<J, String> {
        let start = self.i;
        self.i += 1;
        let mut val = String::new();
        loop {
            if self.i >= self.b.len() {
                return Err("eof in str".into());
            }
            let c = self.b[self.i];
            if c == b'"' {
                self.i += 1;
                break;
            }
            if c == b'\\' {
                self.i += 1;
                if self.i >= self.b.len() {
                    return Err("eof in esc".into());
                }
                let e = self.b[self.i];
                match e {
                    b'"' => val.push('"'),
                    b'\\' => val.push('\\'),
                    b'/' => val.push('/'),
                    b'b' => val.push('\u{0008}'),
                    b'f' => val.push('\u{000C}'),
                    b'n' => val.push('\n'),
                    b'r' => val.push('\r'),
                    b't' => val.push('\t'),
                    b'u' => {
                        if self.i + 4 >= self.b.len() {
                            return Err("eof in \\u".into());
                        }
                        let hex = std::str::from_utf8(&self.b[self.i + 1..self.i + 5])
                            .map_err(|_| "bad \\u".to_string())?;
                        let mut cp =
                            u32::from_str_radix(hex, 16).map_err(|_| "bad \\u".to_string())?;
                        self.i += 4;
                        if (0xD800..0xDC00).contains(&cp)
                            && self.i + 5 < self.b.len()
                            && self.b[self.i + 1] == b'\\'
                            && self.b[self.i + 2] == b'u'
                        {
                            let hex2 = std::str::from_utf8(&self.b[self.i + 3..self.i + 7])
                                .map_err(|_| "bad \\u2".to_string())?;
                            let lo = u32::from_str_radix(hex2, 16)
                                .map_err(|_| "bad \\u2".to_string())?;
                            if (0xDC00..0xE000).contains(&lo) {
                                cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                                self.i += 6;
                            }
                        }
                        val.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                    }
                    _ => return Err("bad escape".into()),
                }
                self.i += 1;
            } else {
                let len = if c < 0x80 {
                    1
                } else if c >> 5 == 0b110 {
                    2
                } else if c >> 4 == 0b1110 {
                    3
                } else {
                    4
                };
                if self.i + len > self.b.len() {
                    return Err("eof in utf8".into());
                }
                val.push_str(
                    std::str::from_utf8(&self.b[self.i..self.i + len])
                        .map_err(|_| "bad utf8".to_string())?,
                );
                self.i += len;
            }
        }
        let raw =
            String::from_utf8(self.b[start..self.i].to_vec()).map_err(|_| "raw".to_string())?;
        Ok(J::Str { raw, val })
    }
    pub fn parse_num(&mut self) -> Result<J, String> {
        let start = self.i;
        while self.i < self.b.len()
            && matches!(
                self.b[self.i],
                b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E'
            )
        {
            self.i += 1;
        }
        if start == self.i {
            return Err(format!("bad number at {}", start));
        }
        let raw =
            String::from_utf8(self.b[start..self.i].to_vec()).map_err(|_| "raw".to_string())?;
        let ival = raw.parse::<i64>().ok();
        let fval = raw.parse::<f64>().unwrap_or(f64::NAN);
        Ok(J::Num { raw, ival, fval })
    }
}

pub fn emit(v: &J, out: &mut String) {
    match v {
        J::Null => out.push_str("null"),
        J::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        J::Num { raw, .. } => out.push_str(raw),
        J::Str { raw, .. } => out.push_str(raw),
        J::Arr(items) => {
            out.push('[');
            for (i, x) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                emit(x, out);
            }
            out.push(']');
        }
        J::Obj(items) => {
            out.push('{');
            for (i, (k, x)) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('"');
                for c in k.chars() {
                    match c {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                        c => out.push(c),
                    }
                }
                out.push('"');
                out.push(':');
                emit(x, out);
            }
            out.push('}');
        }
    }
}

pub fn jget<'a>(root: &'a J, key: &str) -> Option<&'a J> {
    match root {
        J::Obj(items) => items.iter().find(|(k, _)| k == key).map(|(_, v)| v),
        _ => None,
    }
}
pub fn jget_mut<'a>(root: &'a mut J, key: &str) -> Option<&'a mut J> {
    match root {
        J::Obj(items) => items.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v),
        _ => None,
    }
}
pub fn as_arr(v: &J) -> Option<&Vec<J>> {
    match v {
        J::Arr(a) => Some(a),
        _ => None,
    }
}
pub fn as_arr_mut(v: &mut J) -> Option<&mut Vec<J>> {
    match v {
        J::Arr(a) => Some(a),
        _ => None,
    }
}
pub fn as_i64(v: &J) -> Option<i64> {
    match v {
        J::Num { ival: Some(i), .. } => Some(*i),
        J::Num { fval, .. } => Some(*fval as i64),
        _ => None,
    }
}
pub fn as_f64(v: &J) -> Option<f64> {
    match v {
        J::Num { fval, .. } => Some(*fval),
        _ => None,
    }
}
pub fn set_int(v: &mut J, n: i64) {
    if let J::Num { raw, ival, fval } = v {
        *raw = n.to_string();
        *ival = Some(n);
        *fval = n as f64;
    }
}
pub fn set_float(v: &mut J, n: f64) {
    if let J::Num { raw, ival: _, fval } = v {
        *raw = format!("{}", n);
        *fval = n;
    }
}
pub fn set_bool(v: &mut J, b: bool) {
    if let J::Bool(x) = v {
        *x = b;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_primitives() {
        let mut p = Parser { b: b"true", i: 0 };
        assert!(matches!(p.parse_value().unwrap(), J::Bool(true)));

        let mut p = Parser { b: b"false", i: 0 };
        assert!(matches!(p.parse_value().unwrap(), J::Bool(false)));

        let mut p = Parser { b: b"null", i: 0 };
        assert!(matches!(p.parse_value().unwrap(), J::Null));

        let mut p = Parser { b: b"12345", i: 0 };
        let v = p.parse_value().unwrap();
        assert_eq!(as_i64(&v), Some(12345));

        let mut p = Parser { b: b"123.45", i: 0 };
        let v = p.parse_value().unwrap();
        assert_eq!(as_f64(&v), Some(123.45));
    }

    #[test]
    fn test_parse_and_emit_object() {
        let raw = r#"{"name":"test","count":42,"active":true,"tags":[1,2,3]}"#;
        let mut p = Parser {
            b: raw.as_bytes(),
            i: 0,
        };
        let v = p.parse_value().unwrap();

        assert_eq!(as_i64(jget(&v, "count").unwrap()), Some(42));
        assert!(matches!(jget(&v, "active").unwrap(), J::Bool(true)));
        assert_eq!(as_arr(jget(&v, "tags").unwrap()).unwrap().len(), 3);

        let mut out = String::new();
        emit(&v, &mut out);
        assert_eq!(out, raw);
    }

    #[test]
    fn test_jget() {
        let mut obj = J::Obj(vec![
            ("key1".to_string(), J::Bool(true)),
            (
                "key2".to_string(),
                J::Num {
                    raw: "123".to_string(),
                    ival: Some(123),
                    fval: 123.0,
                },
            ),
        ]);

        assert!(matches!(jget(&obj, "key1"), Some(J::Bool(true))));
        assert!(matches!(jget(&obj, "key2"), Some(J::Num { .. })));
        assert!(jget(&obj, "key3").is_none());

        let non_obj = J::Arr(vec![]);
        assert!(jget(&non_obj, "key1").is_none());

        if let Some(J::Bool(b)) = jget_mut(&mut obj, "key1") {
            *b = false;
        }
        assert!(matches!(jget(&obj, "key1"), Some(J::Bool(false))));
        assert!(jget_mut(&mut obj, "key3").is_none());
    }

    #[test]
    fn test_parse_arr() {
        let raw = r#"[1, "two", true, null, [3, 4], {}]"#;
        let mut p = Parser {
            b: raw.as_bytes(),
            i: 0,
        };
        let v = p.parse_value().unwrap();

        let arr = as_arr(&v).unwrap();
        assert_eq!(arr.len(), 6);
        assert_eq!(as_i64(&arr[0]), Some(1));

        match &arr[1] {
            J::Str { val, .. } => assert_eq!(val, "two"),
            _ => panic!("Expected string"),
        }

        assert!(matches!(arr[2], J::Bool(true)));
        assert!(matches!(arr[3], J::Null));

        let nested = as_arr(&arr[4]).unwrap();
        assert_eq!(nested.len(), 2);
        assert_eq!(as_i64(&nested[0]), Some(3));
        assert_eq!(as_i64(&nested[1]), Some(4));
        assert!(matches!(arr[5], J::Obj(_)));
    }

    #[test]
    fn test_parse_str() {
        let raw = r#""hello world""#;
        let mut p = Parser {
            b: raw.as_bytes(),
            i: 0,
        };
        match p.parse_value().unwrap() {
            J::Str { val, .. } => assert_eq!(val, "hello world"),
            _ => panic!("Expected string"),
        }

        let raw = r#""\"\\/\b\f\n\r\t""#;
        let mut p = Parser {
            b: raw.as_bytes(),
            i: 0,
        };
        match p.parse_value().unwrap() {
            J::Str { val, .. } => assert_eq!(val, "\"\\/\x08\x0C\n\r\t"),
            _ => panic!("Expected string"),
        }

        let raw = r#""\u0041\u2603""#;
        let mut p = Parser {
            b: raw.as_bytes(),
            i: 0,
        };
        match p.parse_value().unwrap() {
            J::Str { val, .. } => assert_eq!(val, "A☃"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_parse_num() {
        let cases: Vec<(&str, Option<i64>, f64)> = vec![
            ("0", Some(0), 0.0),
            ("123", Some(123), 123.0),
            ("-123", Some(-123), -123.0),
            ("12.34", None, 12.34),
            ("-12.34", None, -12.34),
            ("1e3", None, 1000.0),
            ("1.5e2", None, 150.0),
        ];

        for (raw, exp_ival, exp_fval) in cases {
            let mut p = Parser {
                b: raw.as_bytes(),
                i: 0,
            };
            let v = p.parse_value().unwrap();
            match v {
                J::Num { ival, fval, .. } => {
                    assert_eq!(ival, exp_ival, "failed ival on {}", raw);
                    assert!((fval - exp_fval).abs() < 1e-6, "failed fval on {}", raw);
                }
                _ => panic!("Expected Num for {}", raw),
            }
        }
    }
}
