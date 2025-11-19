use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

pub const WORDS: &[&str] = &[
    "the", "be", "of", "and", "a", "to", "in", "he", "have", "it", "that", "for", "they", "i",
    "with", "as", "not", "on", "she", "at", "by", "this", "we", "you", "do", "but", "from", "or",
    "which", "one", "would", "all", "will", "there", "say", "who", "make", "when", "can", "more",
    "if", "no", "man", "out", "other", "so", "what", "time", "up", "go", "about", "than", "into",
    "could", "state", "only", "new", "year", "some", "take", "come", "these", "know", "see", "use",
    "get", "like", "then", "first", "any", "work", "now", "may", "such", "give", "over", "think",
    "most", "even", "find", "day", "also", "after", "way", "many", "must", "look", "before",
    "great", "back", "through", "long", "where", "much", "should", "well", "people", "down", "own",
    "just", "because", "good", "each", "those", "feel", "seem", "how", "high", "too", "place",
    "little", "world", "very", "still", "nation", "hand", "old", "life", "tell", "write", "become",
    "here", "show", "house", "both", "between", "need", "mean", "call", "develop", "under",
    "last", "right", "move", "thing", "general", "school", "never", "same", "another", "begin",
    "while", "number", "part", "turn", "real", "leave", "might", "want", "point", "form", "off",
    "child", "few", "small", "since", "against", "ask", "late", "home", "interest", "large",
    "person", "end", "open", "public", "follow", "during", "present", "without", "again", "hold",
    "govern", "around", "possible", "head", "consider", "word", "program", "problem", "however",
    "lead", "system", "set", "order", "eye", "plan", "run", "keep", "face", "fact", "group",
    "play", "stand", "increase", "early", "course", "change", "help", "line",
];

pub fn get_random_words(count: usize, punctuation: bool, numbers: bool) -> Vec<String> {
    let mut rng = thread_rng();
    let mut words: Vec<String> = WORDS
        .choose_multiple(&mut rng, count)
        .map(|&s| s.to_string())
        .collect();

    if numbers {
        for word in words.iter_mut() {
            if rng.gen_bool(0.1) { // 10% chance to be a number
                *word = rng.gen_range(0..1000).to_string();
            }
        }
    }

    if punctuation {
        let puncts = [".", ",", "!", "?", ";", ":"];
        for word in words.iter_mut() {
            if rng.gen_bool(0.2) { // 20% chance to have punctuation
                let p = puncts.choose(&mut rng).unwrap();
                word.push_str(p);
                // Capitalize next word if it's a sentence ender (simplified: just capitalize this one if needed or next? 
                // For simplicity in a type test, we usually just append punctuation. 
                // Real sentence structure is harder. Let's just append for now.)
            }
        }
        // Capitalize first word
        if let Some(first) = words.first_mut() {
            // Capitalize logic
             let mut c = first.chars();
             match c.next() {
                 None => String::new(),
                 Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
             };
        }
    }
    
    words
}
