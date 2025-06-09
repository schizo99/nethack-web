use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use vte::{Parser, Perform};

const WIDTH: usize = 160;
const HEIGHT: usize = 24;

#[derive(Debug)]
struct TtyrecRecord {
    data: Vec<u8>,
}

#[derive(Debug)]
struct Screen {
    rows: Vec<Vec<char>>,
    cursor_x: usize,
    cursor_y: usize,
}

impl Screen {
    fn new() -> Self {
        Self {
            rows: vec![vec![' '; WIDTH]; HEIGHT],
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    fn reset(&mut self) {
        self.rows = vec![vec![' '; WIDTH]; HEIGHT];
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    fn get_row(&self, row: usize) -> Option<String> {
        self.rows
            .get(row)
            .map(|r| r.iter().collect::<String>().trim_end().to_string())
    }
}

impl Perform for Screen {
    fn print(&mut self, c: char) {
        if self.cursor_y < HEIGHT && self.cursor_x < WIDTH {
            self.rows[self.cursor_y][self.cursor_x] = c;
            self.cursor_x += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.cursor_y = (self.cursor_y + 1).min(HEIGHT - 1);
                self.cursor_x = 0;
            }
            b'\r' => self.cursor_x = 0,
            b'\x0c' => self.reset(),
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        match action {
            'H' | 'f' => {
                let y = params
                    .iter()
                    .nth(0)
                    .and_then(|v| v.first())
                    .copied()
                    .unwrap_or(1) as usize;
                let x = params
                    .iter()
                    .nth(1)
                    .and_then(|v| v.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.cursor_x = x.saturating_sub(1);
                self.cursor_y = y.saturating_sub(1);
            }
            'J' => self.reset(),
            _ => {}
        }
    }
}

fn parse_ttyrec<P: AsRef<Path>>(path: P) -> io::Result<Vec<TtyrecRecord>> {
    let mut file = File::open(path)?;
    let mut records = Vec::new();

    loop {
        let mut header = [0u8; 12];
        if file.read_exact(&mut header).is_err() {
            break;
        }

        let len = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
        let mut data = vec![0u8; len];
        file.read_exact(&mut data)?;
        records.push(TtyrecRecord { data });
    }

    Ok(records)
}

fn looks_useful(row: &str) -> bool {
    let trimmed = row.trim();
    let lower = trimmed.to_lowercase();

    if trimmed.is_empty() || trimmed.len() < 4 {
        return false;
    }

    if trimmed.len() == 1 {
        return false;
    }

    let ignore_headers = [
        "weapons",
        "armor",
        "rings",
        "wands",
        "scrolls",
        "potions",
        "food",
        "tools",
        "miscellaneous",
    ];
    if ignore_headers.contains(&lower.as_str()) {
        return false;
    }

    if trimmed.len() > 3 && trimmed.chars().nth(1) == Some(' ') && trimmed.chars().nth(2) == Some('-') {
        return false;
    }

    if trimmed.contains("[ynq]")
        || trimmed.contains("[cn or ?*]")
        || trimmed.contains("[ynqq]")
        || trimmed.contains("[ynaq]")
        || trimmed.contains("[ynaq?]")
        || trimmed.contains("(q)")
    {
        return false;
    }

    lower.starts_with("you ")
        || lower.starts_with("the ")
        || lower.starts_with("this ")
        || lower.starts_with("there is")
        || lower.contains("you see")
        || lower.contains("you feel")
        || lower.contains("you hear")
        || lower.contains("swap places")
        || lower.contains("want to")
        || lower.contains("eat it")
        || lower.contains("zap what")
        || lower.contains("zap which spell")
        || lower.contains("cast which spell")
        || lower.contains("in which direction")
        || lower.contains("quaff")
        || lower.contains("what do you want")
        || lower.contains("corpse")
        || lower.contains("you don't have")
        || lower.contains("you don't know")
}

fn match_failed_action(row: &str) -> Option<String> {
    let row = row.to_lowercase();
    if row.contains("don't have anything to drink") {
        Some("You tried to drink something you didn't have".to_string())
    } else if row.contains("don't know any spells") {
        Some("You tried to cast a spell, but alas you didn't know any".to_string())
    } else if row.contains("don't have anything to zap") {
        Some("You tried to zap a wand that was not in your inventory".to_string())
    } else {
        None
    }
}

fn describe_single_char_action(c: char) -> Option<&'static str> {
    match c {
        'o' => Some("You try to open something"),
        's' => Some("You spent time searching"),
        'Z' => Some("You cast a spell"),
        'z' => Some("You zap a wand"),
        _ => None,
    }
}

fn parse_status_line(line: &str) -> Option<(String, String)> {
    let hp_prefix = "HP:";
    let pw_prefix = "Pw:";
    let parts: Vec<&str> = line.split_whitespace().collect();

    let mut hp_current = None;
    let mut hp_max = None;
    let mut pw_current = None;
    let mut pw_max = None;

    for part in parts {
        if part.starts_with(hp_prefix) {
            let val = part.trim_start_matches(hp_prefix);
            let mut split = val.split(['(', ')'].as_ref());
            hp_current = split.next().map(|s| s.to_string());
            hp_max = split.next().map(|s| s.to_string());
        } else if part.starts_with(pw_prefix) {
            let val = part.trim_start_matches(pw_prefix);
            let mut split = val.split(['(', ')'].as_ref());
            pw_current = split.next().map(|s| s.to_string());
            pw_max = split.next().map(|s| s.to_string());
        }
    }

    if let (Some(hp_c), Some(hp_m), Some(pw_c), Some(pw_m)) =
        (hp_current, hp_max, pw_current, pw_max)
    {
        Some((format!("{}/{}", hp_c, hp_m), format!("{}/{}", pw_c, pw_m)))
    } else {
        None
    }
}

fn parse_turn(line: &str) -> Option<usize> {
    for part in line.split_whitespace() {
        if let Some(stripped) = part.strip_prefix("T:") {
            if let Ok(turn_num) = stripped.parse::<usize>() {
                return Some(turn_num);
            }
        }
    }
    None
}

fn parse_identity(line: &str) -> Option<(String, String)> {
    // Parse username and class from a line like:
    // "Antihumor the Evoker           St:11 Dx:11 ..."
    let words: Vec<&str> = line.split_whitespace().collect();
    if words.len() < 3 {
        return None;
    }
    if let Some(the_index) = words.iter().position(|&w| w == "the") {
        if the_index > 0 && the_index + 1 < words.len() {
            let username = words[..the_index].join(" ");
            let class = words[the_index + 1].to_string();
            return Some((username, class));
        }
    }
    None
}

fn clean_line(line: &str) -> String {
    line.replace("--More--", "")
        .trim_end_matches('-')
        .trim()
        .to_string()
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file.ttyrec>", args[0]);
        std::process::exit(1);
    }

    let ttyrec_path = &args[1];
    let records = parse_ttyrec(ttyrec_path)?;
    let mut screen = Screen::new();
    let mut parser = Parser::new();

    let log_path = std::env::current_dir()?.join("ttyrec_log.txt");

    let mut output_lines = Vec::new();

    let mut current_hp = "Unknown".to_string();
    let mut current_pw = "Unknown".to_string();
    let mut current_turn = 0;

    let mut current_username = "Unknown".to_string();
    let mut current_class = "Unknown".to_string();

    for rec in records {
        screen.reset();
        for byte in &rec.data {
            parser.advance(&mut screen, *byte);
        }

        // Parse HP/Pw from rows 20..=23
        let mut found_hp_pw = None;
        for row in 20..=23 {
            if let Some(line) = screen.get_row(row) {
                if let Some(parsed) = parse_status_line(&line) {
                    found_hp_pw = Some(parsed);
                    break;
                }
            }
        }

        if let Some((hp, pw)) = found_hp_pw {
            current_hp = hp;
            current_pw = pw;
        }

        // Parse turn number from row 23 (or wherever it appears)
        if let Some(line) = screen.get_row(23) {
            if let Some(turn) = parse_turn(&line) {
                current_turn = turn;
            }
        }

        // Parse identity (username, class) from row 22 or similar
        if let Some(line) = screen.get_row(22) {
            if let Some((username, class)) = parse_identity(&line) {
                current_username = username;
                current_class = class;
            }
        }

        let prefix = format!(
            "[Turn: {}, HP: {}, Pw: {}, class: {}, user: {}] ",
            current_turn, current_hp, current_pw, current_class, current_username,
        );

        // Detect input and add user action line to output_lines
        if rec.data.len() == 1 {
            let c = rec.data[0] as char;
            if matches!(c, 'o' | 's' | 'Z' | 'z') {
                if let Some(desc) = describe_single_char_action(c) {
                    output_lines.push(format!("{}{}", prefix, desc));
                }
            }
        }

        if let Some(mut line) = screen.get_row(0) {
            line = clean_line(&line);
            if line.is_empty() || should_exclude_line(&line) {
                continue;
            }

            if let Some(failure_msg) = match_failed_action(&line) {
                output_lines.push(format!("{}{}", prefix, failure_msg));
                continue;
            }

            if looks_useful(&line) {
                output_lines.push(format!("{}{}", prefix, line));
            }
        }
    }

    // Remove duplicates while preserving order
    let mut seen = HashSet::new();
    output_lines.retain(|line| seen.insert(line.clone()));

    let mut log = File::create(&log_path)?;
    for line in output_lines {
        writeln!(log, "{}", line)?;
    }

    println!("✅ Log written to {}", log_path.display());
    Ok(())
}

fn should_exclude_line(line: &str) -> bool {
    let patterns = [
        "What do you want to use or apply?",
        // Add more patterns here if needed
    ];
    patterns.iter().any(|p| line.contains(p))
}
