mod trie;
mod data;
use trie::{Trie, TrieNode};
use data::SCRABBLE_DICTIONARY;
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
    if !legal_words.is_prefix(prefix) {
        return;
    }

    if found_words.is_prefix(prefix) {
        return;
    }

    if legal_words.is_word(prefix) && prefix.len() >= 3 {
        found_words.insert(prefix);
        word_set.insert(String::from(prefix));
    }

    let remaining_letters_string = String::from(remaining_letters);
    let vec: Vec<&str> = remaining_letters_string.split(",").collect();
    for (i, &_item) in vec.iter().enumerate() {
        let mut tvec: Vec<&str> = remaining_letters_string.split(",").collect();
        tvec.remove(i);
        let letter_removed = tvec.join(",");
        if vec[i].chars().count() == 1 {
            find_words(direction, legal_words, found_words, word_set, &(String::from(prefix) + vec[i]), &letter_removed);
        } else {
            match direction {
                SolveDirection::Horizontal => {
                    if vec[i].contains("-") {
                        find_words(direction, legal_words, found_words, word_set, &(String::from(prefix) + &vec[i].split("-").collect::<Vec<&str>>().join("")), &letter_removed);
                    } else if vec[i].contains("|") {
                        let letters: Vec<&str> = vec[i].split("|").collect();
                        for letter in &letters {
                            find_words(direction, legal_words, found_words, word_set, &(String::from(prefix) + letter), &letter_removed);
                        }
                    }
                },
                SolveDirection::Vertical => {
                    if vec[i].contains("|") {
                        find_words(direction, legal_words, found_words, word_set, &(String::from(prefix) + &vec[i].split("|").collect::<Vec<&str>>().join("")), &letter_removed);
                    } else if vec[i].contains("-") {
                        let letters: Vec<&str> = vec[i].split("-").collect();
                        for letter in &letters {
                            find_words(direction, legal_words, found_words, word_set, &(String::from(prefix) + letter), &letter_removed);
                        }
                    }
                }
            }
        }
    }
}

#[wasm_bindgen]
pub fn run(letters: &str) -> String {
    // Create Trie
    let mut legal_words = Trie {
        head: TrieNode {
            letter: ' ',
            end_of_word: false,
            nodes: Default::default() // array of None
        }
    };

    read_dict(&mut legal_words);
    println!("{}", legal_words.word_count());
    
    let mut found_words_horiz = Trie {
        head: TrieNode {
            letter: ' ',
            end_of_word: false,
            nodes: Default::default() // array of None
        }
    };

    let mut found_words_vert = Trie {
        head: TrieNode {
            letter: ' ',
            end_of_word: false,
            nodes: Default::default() // array of None
        }
    };

    let mut word_set: BTreeSet<String> = BTreeSet::new();
    find_words(SolveDirection::Horizontal, &legal_words, &mut found_words_horiz, &mut word_set, "", letters);
    find_words(SolveDirection::Vertical, &legal_words, &mut found_words_vert, &mut word_set, "", letters);

    let mut word_vect: Vec<String> = Vec::with_capacity(word_set.len());
    for word in &word_set {
        word_vect.push(word.to_string());
    } 

    word_vect.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));
    return String::from(word_vect.join(","))
}