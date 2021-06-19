mod trie;
mod dictionary;
use trie::Trie;
use dictionary::SCRABBLE_DICTIONARY;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

extern crate console_error_panic_hook;

#[derive(Clone, Copy, PartialEq)]
enum SolveDirection {
    Horizontal,
    Vertical
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
enum LetterType {
    Single,
    Horizontal,
    Vertical
}

#[derive(Clone, Serialize, Deserialize)]
struct LetterBox {
    ltype: LetterType,
    letters: [String; 2],
    pos: u8
}

#[derive(Serialize, Deserialize)]
struct WordResult {
    combination: Vec<LetterBox>,
    word: String,
    levinword: String
}

fn read_dict(trie: &mut Trie){
    for word in SCRABBLE_DICTIONARY {
        trie.insert(word);
    }
}

fn levinchar(lbox: &LetterBox, prefix_map: &mut HashMap<String, String>) -> String {
    let sstring = serde_json::to_string(lbox).unwrap();
    if !prefix_map.contains_key(&sstring) {
        prefix_map.insert(sstring.clone(), (((prefix_map.len() as u8) + 97) as char).to_string());
    }

    return prefix_map[&sstring].clone();
}

fn find_words(direction: SolveDirection, legal_words: &Trie, found_words: &mut Trie, word_vec: &mut Vec<WordResult>, prefix: &str, prefix_vec: &mut Vec<LetterBox>, remaining_letters: &str, prefix_lev: &str, prefix_map: &mut HashMap<String, String>) {
    // If not a prefix of a word, don't explore any further
    if !legal_words.is_prefix(prefix) {
        return;
    }

    // If we've already found this word, don't explore any further
    if found_words.is_prefix(prefix) {
        return;
    }

    // If this is a word and it's 7 characters, insert it
    if legal_words.is_word(prefix) && prefix.len() == 7 {
        found_words.insert(prefix);
        word_vec.push(WordResult {
            combination: prefix_vec.clone(),
            word: prefix.to_string(),
            levinword: prefix_lev.to_string()
        });

        let lastLetter = prefix_vec.last().unwrap();
        if lastLetter.ltype == LetterType::Single ||
            (direction == SolveDirection::Horizontal && lastLetter.ltype == LetterType::Vertical) ||
            (direction == SolveDirection::Vertical && lastLetter.ltype == LetterType::Horizontal) {
            
            let cutWord = &prefix[..prefix.len() - 1];
            if prefix[prefix.len() - 1..] == "s".to_string() && legal_words.is_word(&cutWord) {
                word_vec.push(WordResult {
                    combination: prefix_vec[..prefix_vec.len() - 1].to_vec(),
                    word: cutWord.to_string(),
                    levinword: prefix_lev.to_string()
                });
            }
        }
    }

    // For each letter we have left to explore,
    let tmp_rem_let = String::from(remaining_letters);
    let rl_vec: Vec<&str> = tmp_rem_let.split(",").collect();
    for (i, &_item) in rl_vec.iter().enumerate() {
        // Remove it from the letters remaining and explore it recursively
        let mut tvec: Vec<&str> = rl_vec.clone();
        let letter = tvec.remove(i);
        let newstr = tvec.join(",");
        if letter.chars().count() == 1 {
            let lbox = LetterBox {
                ltype: LetterType::Single,
                letters: [letter.to_string(), "".to_string()],
                pos: 0
            };

            let lchar = levinchar(&lbox, prefix_map);
            prefix_vec.push(lbox);

            find_words(direction, legal_words, found_words, word_vec, &(String::from(prefix) + letter), prefix_vec, &newstr, &(String::from(prefix_lev) + &lchar), prefix_map);
            prefix_vec.pop();
        } else {
            // If this is a 2x1 or 1x2 block, explore it based on the SolveDirection
            let mut solve_strategy = ["-", "|"];
            let mut solve_strategy_e = [LetterType::Horizontal, LetterType::Vertical];
            match direction {
                SolveDirection::Horizontal => {
                    // Default
                },
                SolveDirection::Vertical => {
                    solve_strategy.reverse();
                    solve_strategy_e.reverse();
                }
            }
            
            if letter.contains(solve_strategy[0]) {
                let letters: Vec<&str> = letter.split(solve_strategy[0]).collect();
                assert_eq!(letters.len(), 2);
                let lbox = LetterBox {
                    ltype: solve_strategy_e[0],
                    letters: [letters[0].to_string(), letters[1].to_string()],
                    pos: 0
                };

                let lchar = levinchar(&lbox, prefix_map);
                prefix_vec.push(lbox);
                
                find_words(direction, legal_words, found_words, word_vec, &(String::from(prefix) + &letters.join("")), prefix_vec, &newstr, &(String::from(prefix_lev) + &lchar), prefix_map);
                prefix_vec.pop();
            } else if letter.contains(solve_strategy[1]) {
                let letters: Vec<&str> = letter.split(solve_strategy[1]).collect();
                assert_eq!(letters.len(), 2);
                for (i, &letter) in letters.iter().enumerate() {
                    let lbox = LetterBox {
                        ltype: solve_strategy_e[1],
                        letters: [letters[0].to_string(), letters[1].to_string()],
                        pos: i as u8
                    };

                    let lchar = levinchar(&lbox, prefix_map);
                    prefix_vec.push(lbox);

                    find_words(direction, legal_words, found_words, word_vec, &(String::from(prefix) + letter), prefix_vec, &newstr, &(String::from(prefix_lev) + &lchar), prefix_map);
                    prefix_vec.pop();
                }
            }
        }
    }
}

#[wasm_bindgen]
pub fn run(letters: &str) -> wasm_bindgen::JsValue {
    console_error_panic_hook::set_once();
    // Create Trie with scrabble dictionary
    let mut legal_words = Trie::new();
    read_dict(&mut legal_words);
    
    // Find words
    let mut found_words_horiz = Trie::new();
    let mut found_words_vert = Trie::new();
    let mut word_vec: Vec<WordResult> = Vec::new();
    let mut prefix_vec: Vec<LetterBox> = Vec::new();
    let mut prefix_map: HashMap<String, String> = HashMap::new();
    find_words(SolveDirection::Horizontal, &legal_words, &mut found_words_horiz, &mut word_vec, "", &mut prefix_vec, letters, "", &mut prefix_map);
    //find_words(SolveDirection::Vertical, &legal_words, &mut found_words_vert, &mut word_vec, "", &mut prefix_vec, letters);

    JsValue::from_serde(&word_vec).unwrap()
}
