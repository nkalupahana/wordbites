const ALPHABET_LENGTH:usize = 26;

fn first_char_idx(word: &str) -> usize {
    return (word.as_bytes()[0] - 97) as usize;
}

pub struct TrieNode {
    pub letter: char,
    pub end_of_word: bool,
    pub nodes: [Option<Box<TrieNode>>; ALPHABET_LENGTH]
}

impl TrieNode {
    pub fn insert(&mut self, word: &str) {
        match word.chars().nth(0) {
            Some(letter) => {
                let idx = first_char_idx(word);
                if self.nodes[idx].is_none() {
                    self.nodes[idx] = Some(Box::new(TrieNode {
                        letter,
                        end_of_word: false,
                        nodes: Default::default() // array of None
                    }));
                }
        
                self.nodes[idx].as_mut().unwrap().insert(&word[1..]);
            },
            None => {
                self.end_of_word = true;
                return;
            }
        }
    }
    
    pub fn is_word(&self, word: &str) -> bool {
        if word.is_empty() {
            return self.end_of_word; 
        }
        
        let idx = first_char_idx(word);
        
        match &self.nodes[idx] {
            None => false,
            Some(node) => {
                return node.is_word(&word[1..]);
            }
        }
    }
    
    pub fn is_prefix(&self, word: &str) -> bool {
        if word.is_empty() {
            return true; 
        }
        
        let idx = first_char_idx(word);
        
        match &self.nodes[idx] {
            None => false,
            Some(node) => {
                return node.is_prefix(&word[1..]);
            }
        }
    }
    
    pub fn word_count(&self) -> i32 {
        let mut sum = self.end_of_word as i32;
        for element in self.nodes.iter() {
            match &element {
                Some(node) => {
                    sum += node.word_count();
                },
                None => ()
            }
        }
        
        return sum;
    }
}

pub struct Trie {
    pub head: TrieNode
}

impl Trie {
    pub fn insert(&mut self, word: &str) {
        return self.head.insert(&word);
    }
    
    pub fn is_word(&self, word: &str) -> bool {
        return self.head.is_word(&word);
    }
    
    pub fn is_prefix(&self, word: &str) -> bool {
        if (word.chars().count() == 0 && self.word_count() == 0) {
            return false;
        }
        
        return self.head.is_prefix(&word);
    }
    
    pub fn word_count(&self) -> i32 {
        return self.head.word_count();
    }
}