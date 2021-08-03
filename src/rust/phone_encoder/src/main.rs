use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::Path;

type Dictionary = HashMap<Vec<u8>, Vec<String>>;

/// Port of Peter Norvig's Lisp solution to the Prechelt phone-encoding problem.
///
/// Even though this is intended as a port, it deviates quite a bit from it
/// due to the very different natures of Lisp and Rust.
fn main() -> io::Result<()> {
    // drop itself from args
    let mut args = args().skip(1);
    let words_file: String = args.next().unwrap_or("tests/words.txt".into());
    let input_file: String = args.next().unwrap_or("tests/numbers.txt".into());

    let dict = load_dict(words_file)?;

    // pre-lock stdout and use buffered output
    let stdout = std::io::stdout();
    let lock = stdout.lock();
    let mut buf = BufWriter::new(lock);

    for line in read_lines(input_file)? {
        let num = line?;
        let digits: Vec<_> = num.chars()
            .filter_map(numeric_char_to_digit)
            .collect();
        write_translations(&mut buf, &dict, &digits, false, &mut |writer| {
            writer.write(num.as_bytes())?;
            writer.write(":".as_bytes())?;
            Ok(())
        })?;
    }
    Ok(())
}

fn write_translations<'dict, W: Write>(
    writer: &mut W,
    dict: &'dict Dictionary,
    digits: &[u8],
    after_digit: bool,
    prefix: &mut dyn FnMut(&mut W) -> io::Result<()>,
) -> io::Result<()> {
    if digits.len() == 0 {
        prefix(writer)?;
        writer.write("\n".as_bytes())?;
    } else {
        let mut found_word = false;
        for i in 0..digits.len() {
            let (n, rest) = digits.split_at(i + 1);
            if let Some(ws) = dict.get(n) {
                for word in ws {
                    found_word = true;
                    write_translations(
                        writer,
                        dict,
                        rest,
                        false,
                        &mut |writer| {
                            prefix(writer)?;
                            writer.write(" ".as_bytes())?;
                            writer.write(word.as_bytes())?;
                            Ok(())
                        })?;
                }
            }
        }
        if !found_word && !after_digit {
            write_translations(
                writer,
                dict,
                &digits[1..],
                true,
                &mut |writer| {
                    prefix(writer)?;
                    writer.write(" ".as_bytes())?;
                    writer.write(digit_to_str(digits[0]).as_bytes())?;
                    Ok(())
                },
            )?;
        }
    }
    Ok(())
}

fn digit_to_str(digit: u8) -> &'static str {
    &"0123456789"[digit as usize..][..1]
}

fn load_dict<P>(words_file: P) -> io::Result<Dictionary>
where P: AsRef<Path> {
    let mut dict = HashMap::with_capacity(100);
    let words = read_lines(words_file)?;
    for line in words {
        if let Ok(word) = line {
            let key = word_to_number(&word);
            let words = dict.entry(key).or_insert_with(|| Vec::new());
            words.push(word);
        }
    }
    Ok(dict)
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn word_to_number(word: &str) -> Vec<u8> {
    word.chars().filter_map(alpha_char_to_digit).collect()
}

fn alpha_char_to_digit(ch: char) -> Option<u8> {
    Some(match ch.to_ascii_lowercase() {
        'e' => 0,
        'j' | 'n' | 'q' => 1,
        'r' | 'w' | 'x' => 2,
        'd' | 's' | 'y' => 3,
        'f' | 't' => 4,
        'a' | 'm' => 5,
        'c' | 'i' | 'v' => 6,
        'b' | 'k' | 'u' => 7,
        'l' | 'o' | 'p' => 8,
        'g' | 'h' | 'z' => 9,
        _ => return None,
    })
}

fn numeric_char_to_digit(ch: char) -> Option<u8> {
    if ch >= '0' && ch <= '9' {
        Some(((ch as usize) - ('0' as usize)) as u8)
    }
    else {
        None
    }
}
