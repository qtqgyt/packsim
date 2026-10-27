#![allow(dead_code)]
#![allow(unused_imports)]

#[path = "../src/json.rs"]
mod json;

use json::{emit, Parser, J};

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        let mut rng = Self { state: seed };
        if rng.state == 0 {
            rng.state = 1;
        }
        for _ in 0..10 {
            rng.next_u64();
        }
        rng
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_f64(&mut self) -> f64 {
        let x = self.next_u64();
        (x >> 11) as f64 / (1u64 << 53) as f64
    }

    fn next_bool(&mut self) -> bool {
        self.next_u64() % 2 == 0
    }

    fn range(&mut self, start: usize, end: usize) -> usize {
        if start >= end {
            return start;
        }
        start + (self.next_u64() as usize % (end - start))
    }

    fn choose<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.range(0, items.len())]
    }
}

fn gen_string(rng: &mut Rng) -> String {
    let len = rng.range(0, 20);
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        let c = match rng.range(0, 100) {
            0..=20 => rng.range(0x20, 0x7E) as u8 as char, // printable ascii
            21..=40 => rng.choose(&['\\', '"', '/', '\x08', '\x0C', '\n', '\r', '\t']).clone(), // control/escaped characters
            41..=80 => char::from_u32(rng.range(0x00A0, 0xD7FF) as u32).unwrap_or('?'), // bmp
            _ => char::from_u32(rng.range(0xE000, 0x10FFFF) as u32).unwrap_or('?'), // high bmp and supplementary
        };
        s.push(c);
    }
    s
}

fn gen_num(rng: &mut Rng) -> J {
    match rng.range(0, 4) {
        0 => { // integer
            let val = rng.next_u64() as i64;
            let raw = val.to_string();
            let fval = val as f64;
            J::Num { raw, ival: Some(val), fval }
        }
        1 => { // floating point
            let mut val = rng.next_f64() * 1e6;
            if rng.next_bool() {
                val = -val;
            }
            let raw = val.to_string();
            J::Num { raw, ival: None, fval: val }
        }
        2 => { // scientific notation
            let mantissa = rng.next_f64() * 10.0;
            let exponent = rng.range(0, 20) as i32 - 10;
            let raw = format!("{}e{}", mantissa, exponent);
            let fval = raw.parse::<f64>().unwrap_or(f64::NAN);
            J::Num { raw, ival: None, fval }
        }
        _ => { // zero
            J::Num { raw: "0".to_string(), ival: Some(0), fval: 0.0 }
        }
    }
}

fn escape_string(s: &str) -> String {
    let mut out = String::new();
    out.push('"');
    for c in s.chars() {
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
    out
}

fn gen_j(rng: &mut Rng, depth: usize) -> J {
    if depth == 0 {
        match rng.range(0, 4) {
            0 => J::Null,
            1 => J::Bool(rng.next_bool()),
            2 => {
                let s = gen_string(rng);
                J::Str { raw: escape_string(&s), val: s }
            },
            _ => gen_num(rng),
        }
    } else {
        match rng.range(0, 6) {
            0 => J::Null,
            1 => J::Bool(rng.next_bool()),
            2 => {
                let s = gen_string(rng);
                J::Str { raw: escape_string(&s), val: s }
            },
            3 => gen_num(rng),
            4 => {
                let len = rng.range(0, 10);
                let mut arr = Vec::with_capacity(len);
                for _ in 0..len {
                    arr.push(gen_j(rng, depth - 1));
                }
                J::Arr(arr)
            }
            _ => {
                let len = rng.range(0, 10);
                let mut obj = Vec::with_capacity(len);
                for _ in 0..len {
                    obj.push((gen_string(rng), gen_j(rng, depth - 1)));
                }
                J::Obj(obj)
            }
        }
    }
}

fn f64_eq(a: f64, b: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        true
    } else {
        (a - b).abs() < 1e-9 || a == b
    }
}

fn j_eq(a: &J, b: &J) -> bool {
    match (a, b) {
        (J::Null, J::Null) => true,
        (J::Bool(b1), J::Bool(b2)) => b1 == b2,
        (J::Num { fval: f1, .. }, J::Num { fval: f2, .. }) => f64_eq(*f1, *f2),
        (J::Str { val: v1, .. }, J::Str { val: v2, .. }) => v1 == v2,
        (J::Arr(a1), J::Arr(a2)) => {
            if a1.len() != a2.len() {
                return false;
            }
            for (v1, v2) in a1.iter().zip(a2.iter()) {
                if !j_eq(v1, v2) {
                    return false;
                }
            }
            true
        }
        (J::Obj(o1), J::Obj(o2)) => {
            if o1.len() != o2.len() {
                return false;
            }
            // we care about order for our emit, but parser doesn't guarantee order if duplicate keys,
            // but we'll assume strict equality including order for now,
            // or we could check keys ignoring order
            for ((k1, v1), (k2, v2)) in o1.iter().zip(o2.iter()) {
                if k1 != k2 || !j_eq(v1, v2) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

#[test]
fn test_json_roundtrip_property() {
    let mut rng = Rng::new(42);

    for i in 0..1000 {
        let ast = gen_j(&mut rng, 3);

        let mut emitted = String::new();
        emit(&ast, &mut emitted);

        let mut parser = Parser { b: emitted.as_bytes(), i: 0 };
        let parsed = match parser.parse_value() {
            Ok(v) => v,
            Err(e) => panic!("Failed to parse emitted JSON on iteration {}: error {}, JSON: {}", i, e, emitted),
        };

        if !j_eq(&ast, &parsed) {
            panic!("Roundtrip mismatch on iteration {}!\nEmitted: {}", i, emitted);
        }
    }
}
