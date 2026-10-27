use std::io::{IsTerminal, Write};

pub fn term_width(width_override: Option<usize>, debug: bool) -> usize {
    let tty = std::io::stderr().is_terminal();
    let ioctl_cols: Option<usize> =
        terminal_size::terminal_size().map(|(terminal_size::Width(w), _)| w as usize);
    let cols_env = std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok());
    let termw: usize = width_override.or(ioctl_cols).or(cols_env).unwrap_or(80);
    if debug {
        eprintln!(
            "[debug] tty={} ioctl={:?} COLUMNS={:?} termw={}",
            tty, ioctl_cols, cols_env, termw
        );
    }
    termw
}

pub fn render_bar(total: u64, count: u64, rate: f64, termw: usize, tty: bool) {
    let pct = if count > 0 {
        total as f64 / count as f64 * 100.0
    } else {
        100.0
    };
    let eta = if rate > 0.0 && total < count {
        let s = ((count - total) as f64 / rate) as u64;
        format!("ETA {:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        "ETA --:--:--".to_string()
    };
    fn hpad(n: f64) -> String {
        format!("{:>9}", human(n))
    }
    if tty {
        let suffix = format!(
            " {:6.2}% {}/{} {}/s {}",
            pct.min(100.0),
            hpad(total as f64),
            hpad(count as f64),
            hpad(rate),
            eta
        );
        let width = termw.saturating_sub(suffix.len() + 4).clamp(10, 400);
        let filled = ((pct / 100.0) * width as f64) as usize;
        let bar: String = std::iter::repeat_n('=', filled.min(width))
            .chain(std::iter::repeat_n(' ', width))
            .take(width)
            .collect();
        eprint!("\r[{}]{}", bar, suffix);
        let _ = std::io::stderr().flush();
    } else {
        eprintln!(
            "progress: {}/{} ({:.2}%) {}/s {}",
            human(total as f64),
            human(count as f64),
            pct.min(100.0),
            human(rate),
            eta
        );
    }
}

pub fn human(n: f64) -> String {
    if n >= 1e15 {
        format!("{:.2}Q", n / 1e15)
    } else if n >= 1e12 {
        format!("{:.2}T", n / 1e12)
    } else if n >= 1e9 {
        format!("{:.2}B", n / 1e9)
    } else if n >= 1e6 {
        format!("{:.2}M", n / 1e6)
    } else if n >= 1e3 {
        format!("{:.2}K", n / 1e3)
    } else {
        format!("{:.0}", n)
    }
}

pub fn money(v: f64) -> String {
    let m = v.abs();
    if m >= 1e15 {
        format!("${:.2}Q", m / 1e15)
    } else if m >= 1e12 {
        format!("${:.2}T", m / 1e12)
    } else if m >= 1e9 {
        format!("${:.2}B", m / 1e9)
    } else if m >= 1e6 {
        format!("${:.2}M", m / 1e6)
    } else if m >= 1e3 {
        format!("${:.2}K", m / 1e3)
    } else {
        format!("${:.2}", m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human() {
        assert_eq!(human(0.0), "0");
        assert_eq!(human(500.0), "500");
        assert_eq!(human(999.0), "999");
        assert_eq!(human(1_000.0), "1.00K");
        assert_eq!(human(1_500.0), "1.50K");
        assert_eq!(human(1_000_000.0), "1.00M");
        assert_eq!(human(1_000_000_000.0), "1.00B");
        assert_eq!(human(1_000_000_000_000.0), "1.00T");
        assert_eq!(human(1_000_000_000_000_000.0), "1.00Q");
    }

    #[test]
    fn test_money() {
        assert_eq!(money(0.0), "$0.00");
        assert_eq!(money(500.5), "$500.50");
        assert_eq!(money(999.0), "$999.00");
        assert_eq!(money(1_000.0), "$1.00K");
        assert_eq!(money(1_500.0), "$1.50K");
        assert_eq!(money(1_000_000.0), "$1.00M");
        assert_eq!(money(1_000_000_000.0), "$1.00B");
        assert_eq!(money(1_000_000_000_000.0), "$1.00T");
        assert_eq!(money(1_000_000_000_000_000.0), "$1.00Q");
    }

    #[test]
    fn test_term_width() {
        assert_eq!(term_width(Some(100), false), 100);
        let tw = term_width(None, false);
        assert!(tw >= 20);
    }
}
