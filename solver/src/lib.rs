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

fn find_words(direction: SolveDirection, legalWords: &Trie, foundWords: &mut Trie, wordSet: &mut BTreeSet<String>, prefix: &str, remainingLetters: &str) {
    if !legalWords.is_prefix(prefix) {
        return;
    }

    if foundWords.is_prefix(prefix) {
        return;
    }

    if legalWords.is_word(prefix) && prefix.len() >= 3 {
        foundWords.insert(prefix);
        wordSet.insert(String::from(prefix));
    }

    let remainingLettersS = String::from(remainingLetters);
    let vec: Vec<&str> = remainingLettersS.split(",").collect();
    for (i, &item) in vec.iter().enumerate() {
        let mut tvec: Vec<&str> = remainingLettersS.split(",").collect();
        tvec.remove(i);
        let letterRemoved = tvec.join(",");
        if vec[i].chars().count() == 1 {
            find_words(direction, legalWords, foundWords, wordSet, &(String::from(prefix) + vec[i]), &letterRemoved);
        } else {
            match direction {
                SolveDirection::Horizontal => {
                    if vec[i].contains("-") {
                        find_words(direction, legalWords, foundWords, wordSet, &(String::from(prefix) + &vec[i].split("-").collect::<Vec<&str>>().join("")), &letterRemoved);
                    } else if vec[i].contains("|") {
                        let mut letters: Vec<&str> = vec[i].split("|").collect();
                        for letter in &letters {
                            find_words(direction, legalWords, foundWords, wordSet, &(String::from(prefix) + letter), &letterRemoved);
                        }
                    }
                },
                SolveDirection::Vertical => {
                    if vec[i].contains("|") {
                        find_words(direction, legalWords, foundWords, wordSet, &(String::from(prefix) + &vec[i].split("|").collect::<Vec<&str>>().join("")), &letterRemoved);
                    } else if vec[i].contains("-") {
                        let mut letters: Vec<&str> = vec[i].split("-").collect();
                        for letter in &letters {
                            find_words(direction, legalWords, foundWords, wordSet, &(String::from(prefix) + letter), &letterRemoved);
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
    let mut legalWords = Trie {
        head: TrieNode {
            letter: ' ',
            end_of_word: false,
            nodes: Default::default() // array of None
        }
    };

    read_dict(&mut legalWords);
    println!("{}", legalWords.word_count());
    
    let mut foundWordsH = Trie {
        head: TrieNode {
            letter: ' ',
            end_of_word: false,
            nodes: Default::default() // array of None
        }
    };

    let mut foundWordsV = Trie {
        head: TrieNode {
            letter: ' ',
            end_of_word: false,
            nodes: Default::default() // array of None
        }
    };

    let mut wordSet: BTreeSet<String> = BTreeSet::new();
    find_words(SolveDirection::Horizontal, &legalWords, &mut foundWordsH, &mut wordSet, "", letters);
    find_words(SolveDirection::Vertical, &legalWords, &mut foundWordsV, &mut wordSet, "", letters);

    let mut wordVect: Vec<String> = Vec::with_capacity(wordSet.len());
    for word in &wordSet {
        wordVect.push(word.to_string());
    } 

    wordVect.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));
    return String::from(wordVect.join(","))
}