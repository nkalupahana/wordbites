mod trie;
mod dictionary;
use trie::Trie;
use dictionary::SCRABBLE_DICTIONARY;
use std::collections::BTreeSet;
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy)]
enum SolveDirection {
    Horizontal,
    Vertical
}

fn read_dict(trie: &mut Trie){
    for word in SCRABBLE_DICTIONARY {
        trie.insert(word);
    }
}

fn find_words(direction: SolveDirection, legal_words: &Trie, found_words: &mut Trie, word_set: &mut BTreeSet<String>, prefix: &str, remaining_letters: &str) {
    // If not a prefix of a word, don't explore any further
    if !legal_words.is_prefix(prefix) {
        return;
    }

    // If we've already found this word, don't explore any further
    if found_words.is_prefix(prefix) {
        return;
    }

    // If this is a word and it's longer than 3 characters, insert it
    if legal_words.is_word(prefix) && prefix.len() >= 3 {
        found_words.insert(prefix);
        word_set.insert(String::from(prefix));
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
            find_words(direction, legal_words, found_words, word_set, &(String::from(prefix) + letter), &newstr);
        } else {
            // If this is a 2x1 or 1x2 block, explore it based on the SolveDirection
            let mut solve_strategy = ["", ""];
            match direction {
                SolveDirection::Horizontal => {
                    solve_strategy[0] = "-";
                    solve_strategy[1] = "|";
                },
                SolveDirection::Vertical => {
                    solve_strategy[0] = "|";
                    solve_strategy[1] = "-";
                }
            }
            
            if letter.contains(solve_strategy[0]) {
                find_words(direction, legal_words, found_words, word_set, &(String::from(prefix) + &letter.split(solve_strategy[0]).collect::<Vec<&str>>().join("")), &newstr);
            } else if letter.contains(solve_strategy[1]) {
                let letters: Vec<&str> = letter.split(solve_strategy[1]).collect();
                for letter in &letters {
                    find_words(direction, legal_words, found_words, word_set, &(String::from(prefix) + letter), &newstr);
                }
            }
        }
    }
}

#[wasm_bindgen]
pub fn run(letters: &str) -> String {
    // Create Trie with scrabble dictionary
    let mut legal_words = Trie::new();
    read_dict(&mut legal_words);
    
    // Find words
    let mut found_words_horiz = Trie::new();
    let mut found_words_vert = Trie::new();
    let mut word_set: BTreeSet<String> = BTreeSet::new();
    //find_words(SolveDirection::Horizontal, &legal_words, &mut found_words_horiz, &mut word_set, "", letters);
    find_words(SolveDirection::Vertical, &legal_words, &mut found_words_vert, &mut word_set, "", letters);

    // TreeSet -> Vec -> String, and return
    let mut word_vect: Vec<String> = Vec::with_capacity(word_set.len());
    for word in &word_set {
        word_vect.push(word.to_string());
    } 
    return String::from(word_vect.join(","))
}