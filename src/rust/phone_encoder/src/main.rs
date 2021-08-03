use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use num_bigint::BigUint;

type Dictionary = HashMap<BigUint, Vec<String>>;

struct Cons<'a, T: 'a> {
    data: T,
    next: Option<&'a Cons<'a, T>>,
}

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

    for line in read_lines(input_file)? {
        if let Ok(num) = line {
            let digits: Vec<_> = num.chars()
                .filter_map(numeric_char_to_digit)
                .collect();
            print_translations(&dict, &num, &digits, None);
        }
    }
    Ok(())
}

fn print_translations<'dict>(
    dict: &'dict Dictionary,
    num: &str,
    digits: &[u8],
    words: Option<&Cons<&'dict str>>,
) {
    if digits.len() == 0 {
        print!("{}:", num);
        print_reversed(words);
        println!();
    } else {
        let mut found_word = false;
        let mut n = 1u8.into();
        for i in 0..digits.len() {
            n *= 10u8;
            n += digits[i];
            if let Some(ws) = dict.get(&n) {
                for word in ws {
                    found_word = true;
                    print_translations(
                        dict,
                        num,
                        &digits[(i + 1)..],
                        Some(&Cons {
                            data: &*word,
                            next: words,
                        }),
                    );
                }
            }
        }
        if !found_word
            && !words
                .map(|c| c.data.chars().all(char::is_numeric))
                .unwrap_or(false)
        {
            print_translations(
                dict,
                num,
                &digits[1..],
                Some(&Cons {
                    data: digit_to_str(digits[0]),
                    next: words,
                }),
            );
        }
    }
}

fn print_reversed(words: Option<&Cons<&str>>) {
    if let Some(c) = words {
        print_reversed(c.next);
        print!(" {}", c.data);
    }
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

fn word_to_number(word: &str) -> BigUint {
    let mut n = 1u8.into();
    for digit in word.chars().filter_map(alpha_char_to_digit) {
        n *= 10u8;
        n += digit;
    }
    n
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
