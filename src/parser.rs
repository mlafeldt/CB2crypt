//! Finds codes in free-form text and decrypts or encrypts them.
//!
//! Follows the parser of the original CB2crypt: a code is two 8-digit hex
//! tokens or one 16-digit token, lines are scanned from right to left so that
//! hex-looking words in a description don't eat the code's address, and every
//! code is moved to a line of its own.
//!
//! Like the original, it drops every blank line of the input, and has an
//! option to put one after each run of codes. Unlike the original, and like
//! cb2util, it always puts one before every game title, which is a line in
//! double quotes. So the blank lines of the output follow from the option
//! alone, and turning it off takes them out of a list that had them. Also
//! like cb2util, every game title resets the crypto, so that a V7 game
//! doesn't garble the games after it.
//!
//! Two bugs of the original are not reproduced: a 16-digit code lost its `?`
//! wildcards, and an 8-digit token with wildcards could become an address.

use codebreaker::Codebreaker;

const DIGITS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Decrypt,
    Encrypt,
    /// Tidies up the text without touching the codes.
    Reformat,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Options {
    /// Treats the list as if it started with the common seed code
    /// `B4336FA9 4DFEFB79`.
    pub common_v7: bool,
    /// Separates each run of codes from the text that follows it.
    pub blank_line: bool,
}

#[derive(Debug, Clone, Copy)]
enum Token<'a> {
    Text(&'a str),
    /// Eight hex digits that may turn out to be half of a code.
    Hex {
        s: &'a str,
        wildcard: bool,
    },
    Addr(&'a str),
    Val {
        s: &'a str,
        wildcard: bool,
    },
}

/// Processes all codes in `text` and returns the output lines.
pub fn process(text: &str, mode: Mode, options: Options) -> Vec<String> {
    let fresh = || {
        if options.common_v7 {
            Codebreaker::new_v7()
        } else {
            Codebreaker::new()
        }
    };
    let mut cb = fresh();
    let mut out = Vec::new();
    let mut after_code = false;
    // Whether a blank line is due before the next line of output.
    let mut gap = false;
    let emit = |out: &mut Vec<String>, gap: &mut bool, line: String| {
        if std::mem::take(gap) && !out.is_empty() {
            out.push(String::new());
        }
        out.push(line);
    };

    // A game always stands apart from the one before it, and any other text
    // from the codes before it if the user wants that. Like on the device,
    // the codes of one game don't depend on those of another.
    let emit_text =
        |out: &mut Vec<String>, gap: &mut bool, after_code: &mut bool, cb: &mut Codebreaker, text: String| {
            let after_code = std::mem::take(after_code);
            if is_game_title(&text) {
                *cb = fresh();
                *gap = true;
            }
            *gap |= options.blank_line && after_code;
            emit(out, gap, text);
        };

    for line in text.lines() {
        let tokens = tokenize(line);
        // Blank lines are not carried over: where the output has one is up
        // to the rules below alone, so running it through again with the
        // option turned off takes out what the option put in.
        if tokens.is_empty() {
            continue;
        }
        let mut pending = String::new();
        let mut i = 0;

        while i < tokens.len() {
            match (tokens[i], tokens.get(i + 1)) {
                (Token::Addr(addr), Some(&Token::Val { s, wildcard })) => {
                    if !pending.is_empty() {
                        emit_text(
                            &mut out,
                            &mut gap,
                            &mut after_code,
                            &mut cb,
                            std::mem::take(&mut pending),
                        );
                    }
                    emit(&mut out, &mut gap, crypt(&mut cb, mode, addr, s, wildcard));
                    after_code = true;
                    i += 2;
                }
                (Token::Text(s) | Token::Hex { s, .. } | Token::Addr(s) | Token::Val { s, .. }, _) => {
                    if !pending.is_empty() {
                        pending.push(' ');
                    }
                    pending.push_str(s);
                    i += 1;
                }
            }
        }
        if !pending.is_empty() {
            emit_text(&mut out, &mut gap, &mut after_code, &mut cb, pending);
        }
    }
    out
}

/// Tells a game title like `"TimeSplitters"` from a code description.
fn is_game_title(text: &str) -> bool {
    text.len() >= 2 && text.starts_with('"') && text.ends_with('"')
}

fn tokenize(line: &str) -> Vec<Token<'_>> {
    let is_hex = |s: &str| s.bytes().all(|b| b.is_ascii_hexdigit());
    let is_hex_or_wildcard = |s: &str| s.bytes().all(|b| b.is_ascii_hexdigit() || b == b'?');

    let mut tokens = Vec::new();
    for s in line.split_whitespace() {
        if s.len() == DIGITS && is_hex_or_wildcard(s) {
            tokens.push(Token::Hex {
                s,
                wildcard: s.contains('?'),
            });
        } else if s.len() == DIGITS * 2
            // Sixteen bytes need not be sixteen characters: the middle may
            // fall inside one, and then this is no code.
            && let Some((addr, val)) = s.split_at_checked(DIGITS)
            && is_hex(addr)
            && is_hex_or_wildcard(val)
        {
            tokens.push(Token::Addr(addr));
            tokens.push(Token::Val {
                s: val,
                wildcard: val.contains('?'),
            });
        } else {
            tokens.push(Token::Text(s));
        }
    }

    // Pair up hex tokens from right to left. An address is no longer a hex
    // token, so the token before it can't claim it as a value.
    for i in (1..tokens.len()).rev() {
        if let (
            Token::Hex {
                s: addr,
                wildcard: false,
            },
            Token::Hex { s, wildcard },
        ) = (tokens[i - 1], tokens[i])
        {
            tokens[i - 1] = Token::Addr(addr);
            tokens[i] = Token::Val { s, wildcard };
        }
    }
    tokens
}

fn crypt(cb: &mut Codebreaker, mode: Mode, addr: &str, val: &str, wildcard: bool) -> String {
    let hex = |s| u32::from_str_radix(s, 16).expect("token is 8 hex digits");
    let addr = hex(addr);
    // The value of a wildcard code is unknown, so only its address is
    // meaningful. That works for CB v1, where the address doesn't depend on
    // the value.
    let plain_val = if wildcard { 0 } else { hex(val) };

    let (addr, new_val) = match mode {
        Mode::Decrypt => cb.auto_decrypt_code(addr, plain_val),
        Mode::Encrypt => cb.encrypt_code(addr, plain_val),
        Mode::Reformat => (addr, plain_val),
    };

    if wildcard {
        format!("{addr:08X} {}", val.to_ascii_uppercase())
    } else {
        format!("{addr:08X} {new_val:08X}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str, mode: Mode, common_v7: bool) -> Vec<String> {
        process(
            text,
            mode,
            Options {
                common_v7,
                ..Options::default()
            },
        )
    }

    fn decrypt(text: &str) -> Vec<String> {
        run(text, Mode::Decrypt, false)
    }

    fn encrypt(text: &str) -> Vec<String> {
        run(text, Mode::Encrypt, false)
    }

    // The examples below are taken from the CB2crypt readme.

    #[test]
    fn wildcard_value_passes_through() {
        assert_eq!(decrypt("Slot 1   1A7D6192 0000????"), ["Slot 1", "10754244 0000????"]);
    }

    #[test]
    fn hex_word_in_description_is_not_an_address() {
        assert_eq!(
            run("1st place Set at 99999998   8D39AD57 23332B61", Mode::Decrypt, true),
            ["1st place Set at 99999998", "205EC200 05F5E0FE"]
        );
    }

    #[test]
    fn v1_round_trip() {
        let raw = "2033BA10 FFFFFFFF\n2033BA14 FFFFFFFF";
        let enc = ["2ABB1147 FFFFFFFF", "2AA71147 FFFFFFFF"];
        assert_eq!(encrypt(raw), enc);
        assert_eq!(decrypt(&enc.join("\n")), raw.lines().collect::<Vec<_>>());
    }

    #[test]
    fn v7_round_trip_with_beefcode() {
        let raw = "BEEFC0DE 00000000\n2033BA10 FFFFFFFF\n2033BA14 FFFFFFFF";
        let enc = ["B4336FA9 4DFEFB79", "AE17E302 4AE8B74D", "E8466F61 F9A3B3A8"];
        assert_eq!(encrypt(raw), enc);
        assert_eq!(decrypt(&enc.join("\n")), raw.lines().collect::<Vec<_>>());
    }

    #[test]
    fn common_v7_replaces_the_seed_code() {
        let raw = "2033BA10 FFFFFFFF\n2033BA14 FFFFFFFF";
        let enc = ["AE17E302 4AE8B74D", "E8466F61 F9A3B3A8"];
        assert_eq!(run(raw, Mode::Encrypt, true), enc);
        assert_eq!(
            run(&enc.join("\n"), Mode::Decrypt, true),
            raw.lines().collect::<Vec<_>>()
        );
    }

    #[test]
    fn official_enable_code() {
        let text = "\"Tiger Woods PGA Tour 2006\"\n\
                    Enable Code\n\
                    B4336FA9 4DFEFB79\n\
                    410C34B6 663C5B57\n\
                    1C4D0A53 7FE27DE4\n\
                    C81AC878 32FCD351";
        assert_eq!(
            decrypt(text),
            [
                "\"Tiger Woods PGA Tour 2006\"",
                "Enable Code",
                "BEEFC0DE 00000000",
                "000FFFFE 0000015A",
                "000FFFFF 0000015A",
                "903DD9F8 00832021",
            ]
        );
    }

    #[test]
    fn encrypt_with_given_seed_code() {
        assert_eq!(
            encrypt("BEEFC0DE 00000000\n204D98B0 3F800000"),
            ["B4336FA9 4DFEFB79", "13896CD2 EF2FA92B"]
        );
    }

    #[test]
    fn raw_codes_after_v7_is_turned_off() {
        assert_eq!(
            encrypt("BEEFC0DE 00000000\nFFFFFFFF 00000000"),
            ["B4336FA9 4DFEFB79", "0CF49BDF A4E29C32"]
        );
        // Like on the device, the line after a seed code never starts a code,
        // so the option line only works further down the list.
        assert_eq!(
            decrypt(
                "B4336FA9 4DFEFB79\n\
                 410C34B6 663C5B57\n\
                 V7 encryption off\n\
                 0CF49BDF A4E29C32\n\
                 Always Full Gamebreaker P1\n\
                 204D98B0 3F800000"
            ),
            [
                "BEEFC0DE 00000000",
                "000FFFFE 0000015A",
                "V7 encryption off",
                "FFFFFFFF 00000000",
                "Always Full Gamebreaker P1",
                "204D98B0 3F800000",
            ]
        );
    }

    // Parser behavior beyond the readme.

    #[test]
    fn sixteen_digit_code_is_split() {
        assert_eq!(decrypt("2AFF014C2411FFFF"), ["2043AFCC 2411FFFF"]);
        assert_eq!(decrypt("1A7D61920000????"), ["10754244 0000????"]);
    }

    #[test]
    fn output_is_uppercase() {
        assert_eq!(decrypt("2aff014c 2411ffff"), ["2043AFCC 2411FFFF"]);
        assert_eq!(decrypt("1a7d6192 00ab????"), ["10754244 00AB????"]);
    }

    #[test]
    fn blank_lines_of_the_input_are_dropped() {
        assert_eq!(
            decrypt("\r\n  \t\r\nA\r\n\r\n \r\n2AFF014C 2411FFFF\r\n\r\n"),
            ["A", "2043AFCC 2411FFFF"]
        );
    }

    #[test]
    fn turning_the_option_off_takes_its_blank_lines_out_again() {
        let text = "\"G\"\nA\n2033BA10 FFFFFFFF\nB\n2033BA14 FFFFFFFF\n\"H\"\nC";
        let plain = process(text, Mode::Reformat, Options::default());
        let options = Options {
            blank_line: true,
            ..Options::default()
        };
        let spaced = process(text, Mode::Reformat, options);
        assert_ne!(spaced, plain);
        assert_eq!(process(&spaced.join("\n"), Mode::Reformat, Options::default()), plain);
    }

    #[test]
    fn games_are_separated_by_one_blank_line() {
        let expected = [
            "\"Game 1\"",
            "Money",
            "2ABB1147 FFFFFFFF",
            "",
            "\"Game 2\"",
            "Lives",
            "2AA71147 FFFFFFFF",
        ];
        let game1 = "\"Game 1\"\nMoney\n2033BA10 FFFFFFFF\n";
        let game2 = "\"Game 2\"\nLives\n2033BA14 FFFFFFFF\n";

        for between in ["", "\n", "\n \n\n"] {
            let text = format!("\n{game1}{between}{game2}\n");
            assert_eq!(encrypt(&text), expected, "{between:?}");
            // The option's own blank line doesn't add to it.
            let options = Options {
                blank_line: true,
                ..Options::default()
            };
            assert_eq!(process(&text, Mode::Encrypt, options), expected, "{between:?}");
        }
    }

    #[test]
    fn quoted_text_next_to_a_code_is_a_game_title_too() {
        assert_eq!(
            decrypt("A\n\"B\" 2AFF014C 2411FFFF\n\"C"),
            ["A", "", "\"B\"", "2043AFCC 2411FFFF", "\"C"]
        );
    }

    #[test]
    fn text_around_a_code_gets_its_own_line() {
        assert_eq!(
            decrypt("before 2AFF014C 2411FFFF after  it"),
            ["before", "2043AFCC 2411FFFF", "after it"]
        );
    }

    #[test]
    fn several_codes_on_one_line() {
        assert_eq!(
            decrypt("2AFF014C 2411FFFF 2AFF014C 2411FFFF"),
            ["2043AFCC 2411FFFF", "2043AFCC 2411FFFF"]
        );
    }

    #[test]
    fn odd_hex_token_out_is_the_leftmost() {
        assert_eq!(decrypt("DEADBEEF 2AFF014C 2411FFFF"), ["DEADBEEF", "2043AFCC 2411FFFF"]);
    }

    #[test]
    fn wildcard_is_only_allowed_in_the_value() {
        assert_eq!(decrypt("0000???? 2411FFFF"), ["0000???? 2411FFFF"]);
        assert_eq!(decrypt("0000????2411FFFF"), ["0000????2411FFFF"]);
    }

    #[test]
    fn non_ascii_text_is_left_alone() {
        assert_eq!(decrypt("Münze ∞ 2AFF014C 2411FFFF"), ["Münze ∞", "2043AFCC 2411FFFF"]);
        assert_eq!(decrypt("ääää"), ["ääää"]);
    }

    #[test]
    fn sixteen_bytes_that_are_not_sixteen_characters_are_text() {
        // Byte 8 falls inside the first "€".
        let word = "123456€€abcd";
        assert_eq!(word.len(), 16);
        assert_eq!(decrypt(word), [word]);
    }

    #[test]
    fn empty_input() {
        assert!(decrypt("").is_empty());
        assert!(run(" \n\n", Mode::Encrypt, true).is_empty());
    }

    #[test]
    fn reformat_leaves_codes_alone() {
        assert_eq!(
            run("Slot 1   1a7d6192 0000????\n\n2aff014c2411ffff", Mode::Reformat, true),
            ["Slot 1", "1A7D6192 0000????", "2AFF014C 2411FFFF"]
        );
    }

    #[test]
    fn blank_line_separates_codes_from_the_next_text() {
        let options = Options {
            blank_line: true,
            ..Options::default()
        };
        assert_eq!(
            process(
                "A\n2033BA10 FFFFFFFF\n2033BA14 FFFFFFFF\nB 2033BA10 FFFFFFFF C",
                Mode::Reformat,
                options
            ),
            [
                "A",
                "2033BA10 FFFFFFFF",
                "2033BA14 FFFFFFFF",
                "",
                "B",
                "2033BA10 FFFFFFFF",
                "",
                "C",
            ]
        );
    }

    #[test]
    fn each_game_starts_with_fresh_crypto() {
        let raw = ["\"G\"", "BEEFC0DE 00000000", "2033BA10 FFFFFFFF", "2033BA14 FFFFFFFF"];
        let enc = ["\"G\"", "B4336FA9 4DFEFB79", "AE17E302 4AE8B74D", "E8466F61 F9A3B3A8"];
        fn twice(game: [&str; 4]) -> Vec<&str> {
            [&game[..], &[""], &game[..]].concat()
        }

        assert_eq!(encrypt(&twice(raw).join("\n")), twice(enc));
        assert_eq!(decrypt(&twice(enc).join("\n")), twice(raw));

        // The common V7 seeds apply to every game.
        let raw = ["\"G\"", "2033BA10 FFFFFFFF", "BEEFC0DE 00000000", "2033BA14 FFFFFFFF"];
        let enc = run(&raw.join("\n"), Mode::Encrypt, true);
        assert_eq!(enc[1], "AE17E302 4AE8B74D");
        let two = [&enc[..], &[String::new()], &enc[..]].concat();
        assert_eq!(run(&twice(raw).join("\n"), Mode::Encrypt, true), two);
        assert_eq!(run(&two.join("\n"), Mode::Decrypt, true), twice(raw));
    }
}
