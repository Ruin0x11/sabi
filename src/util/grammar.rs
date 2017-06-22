use regex::Regex;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VerbPerson {
    I,
    You,
    He,
    She,
    It,
    We,
    They,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VerbTense {
    Past,
    Present,
}

use self::VerbPerson::*;

impl VerbPerson {
    pub fn pronoun(&self) -> &str {
        match *self {
            I => "I",
            You => "you",
            He => "he",
            She => "she",
            It => "it",
            We => "we",
            They => "they",
        }
    }

    pub fn is(&self) -> &str {
        match *self {
            I => "am",
            You => "are",
            He => "is",
            She => "is",
            It => "is",
            We => "are",
            They => "are",
        }
    }

    pub fn was(&self) -> &str {
        match *self {
            I => "was",
            You => "were",
            He => "was",
            She => "was",
            It => "was",
            We => "were",
            They => "were",
        }
    }

    pub fn possessive(&self) -> &str {
        match *self {
            I => "my",
            You => "your",
            He => "his",
            She => "her",
            It => "its",
            We => "our",
            They => "their",
        }
    }

    pub fn accusative(&self) -> &str {
        match *self {
            I => "me",
            You => "you",
            He => "him",
            She => "her",
            It => "it",
            We => "us",
            They => "them",
        }
    }
}

fn last_char(s: &str) -> Option<char> {
    if let Some((_, c)) = s.char_indices().rev().nth(0) {
        return Some(c);
    }

    None
}

fn last_two_chars(s: &str) -> Option<&str> {
    if let Some((i, _)) = s.char_indices().rev().nth(1) {
        return Some(&s[i..]);
    }

    None
}

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

macro_rules! try_rule {
    ($name:ident, $from:expr, $to: expr) => {
        if let Some(result) = apply_rule($name, $from, $to) {
            return result;
        }
    }
}

fn apply_rule(base: &str, matching: &str, to: &str) -> Option<String> {
    let re = Regex::new(matching).unwrap();

    if re.find(base).is_some() {
        return Some(re.replace_all(base, to).to_string());
    }

    None
}

pub fn make_plural(name: &str) -> String {
    try_rule!(name, "(?P<a>[cs])h$", "${a}hes");
    try_rule!(name, "(?P<a>[cs]h|[zx])$", "${a}es");
    try_rule!(name, "(?P<a>us)$", "${a}es");
    try_rule!(name, "ss$", "sses");
    try_rule!(name, "s$", "ses");
    try_rule!(name, "$", "s");

    name.to_string()
}

fn is_vowel(ch: char) -> bool {
    match ch {
        'a' | 'e' | 'i' | 'o' | 'u' => true,
        _ => false,
    }
}

pub fn get_article(name: &str) -> &str {
    if name.is_empty() {
        return "";
    }

    let first_char = |s: &str| s.chars().next().unwrap();
    let first = first_char(name);

    if !is_vowel(first) { "a" } else { "an" }
}

pub fn make_count(name: &str, count: u32) -> String {
    if count == 1 {
        name.to_string()
    } else {
        let plural = make_plural(name);

        let prefix = if count == 0 {
            "no".to_string()
        } else {
            count.to_string()
        };

        format!("{} {}", prefix, plural)
    }
}

pub fn make_count_with_article(name: &str, count: u32) -> String {
    if count == 1 {
        format!("{} {}", get_article(name), name)
    } else {
        make_count(name, count)
    }
}

pub fn make_possessive(s: &str) -> String {
    match s {
        "I" => "my".to_string(),
        "you" => "your".to_string(),
        "it" => "its".to_string(),
        "he" => "his".to_string(),
        "she" => "her".to_string(),
        "we" => "our".to_string(),
        "they" => "their".to_string(),
        _ => {
            match last_char(s) {
                Some('s') => format!("{}'", s),
                _ => format!("{}'s", s),
            }
        },
    }
}

fn try_conjugate_preposition(verb: &str, person: VerbPerson, tense: VerbTense) -> Option<String> {
    let re = Regex::new(r"(\w+)\s(.*)$").unwrap();
    let caps_opt = re.captures(verb);

    caps_opt.map(|caps| {
        let verb = caps.get(1).unwrap().as_str();
        let rest = caps.get(2).unwrap().as_str();
        let conj = format!("{} {}", conjugate(verb, person, tense), rest);
        conj
    })
}

fn try_conjugate_special_verb(verb: &str, person: VerbPerson, tense: VerbTense) -> Option<String> {
    if verb == "be" {
        let s = match tense {
            VerbTense::Past => person.was(),
            VerbTense::Present => person.is(),
        };
        return Some(s.to_string());
    }

    if verb == "have" {
        let s = match tense {
            VerbTense::Past => "had",
            VerbTense::Present => {
                match person {
                    He | She | It => "has",
                    _ => "have",
                }
            },
        };
        return Some(s.to_string());
    }

    None
}

fn conjugate_from_infinitive(verb: &str, person: VerbPerson, tense: VerbTense) -> String {
    let ending = match last_two_chars(verb) {
        Some(e) => e.chars().collect::<Vec<char>>(),
        None => return verb.to_string(),
    };

    match tense {
        VerbTense::Past => verb.to_string(),
        VerbTense::Present => {
            match person {
                He | She | It => {
                    if ending[1] == 's' || ending[1] == 'o' || ending[1] == 'x' ||
                        ending[1] == 'z' ||
                        (ending[0] == 's' && ending[1] == 'h') ||
                        (ending[0] == 'c' && ending[1] == 'h')
                    {
                        // hit->hits
                        // miss->misses
                        // bash->bashes...
                        format!("{}es", verb)
                    } else if ending[1] == 'y' {
                        if is_vowel(ending[0]) {
                            // say -> says
                            format!("{}s", verb)
                        } else {
                            // fly -> flies
                            format!("{}ies", &verb[..2])
                        }
                    } else {
                        format!("{}s", verb)
                    }
                },
                I | You | We | They => verb.to_string(),
            }
        },
    }
}

// staight ripoff of Jeff Lait's conjugation algorithm
pub fn conjugate(verb: &str, person: VerbPerson, tense: VerbTense) -> String {
    // Step 1: Check for preposition (spit at -> spits at)
    if let Some(conj) = try_conjugate_preposition(verb, person, tense) {
        return conj;
    }

    // Step 2: Check for nonstandard verbs (be, have)
    if let Some(conj) = try_conjugate_special_verb(verb, person, tense) {
        return conj;
    }

    // Step 3: Build from infinitive
    conjugate_from_infinitive(verb, person, tense)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plural() {
        assert_eq!(&make_plural("cat"), "cats");
        assert_eq!(&make_plural("bus"), "buses");
        assert_eq!(&make_plural("class"), "classes");
        assert_eq!(&make_plural("walrus"), "walruses");
        assert_eq!(&make_plural("ox"), "oxes");
        assert_eq!(&make_plural("dish"), "dishes");
    }

    #[test]
    fn test_count() {
        assert_eq!(&make_count("cat", 0), "no cats");
        assert_eq!(&make_count("cat", 1), "cat");
        assert_eq!(&make_count("cat", 2), "2 cats");
    }

    #[test]
    fn test_count_with_article() {
        assert_eq!(&make_count_with_article("cat", 1), "a cat");
        assert_eq!(&make_count_with_article("archer", 1), "an archer");
    }

    #[test]
    fn test_conjugate() {
        assert_eq!(&conjugate("spit at", VerbPerson::It, VerbTense::Present), "spits at");

        assert_eq!(&conjugate("be", VerbPerson::You, VerbTense::Past), "were");
        assert_eq!(&conjugate("be", VerbPerson::You, VerbTense::Present), "are");
        assert_eq!(&conjugate("be", VerbPerson::It, VerbTense::Past), "was");
        assert_eq!(&conjugate("be", VerbPerson::It, VerbTense::Present), "is");

        assert_eq!(&conjugate("have", VerbPerson::You, VerbTense::Past), "had");
        assert_eq!(&conjugate("have", VerbPerson::You, VerbTense::Present), "have");
        assert_eq!(&conjugate("have", VerbPerson::It, VerbTense::Present), "has");

        assert_eq!(&conjugate("hit", VerbPerson::It, VerbTense::Present), "hits");
        assert_eq!(&conjugate("miss", VerbPerson::It, VerbTense::Present), "misses");
        assert_eq!(&conjugate("bash", VerbPerson::It, VerbTense::Present), "bashes");
        assert_eq!(&conjugate("fly", VerbPerson::It, VerbTense::Present), "flies");
        assert_eq!(&conjugate("say", VerbPerson::It, VerbTense::Present), "says");
        assert_eq!(&conjugate("go", VerbPerson::It, VerbTense::Present), "goes");
        assert_eq!(&conjugate("watch", VerbPerson::It, VerbTense::Present), "watches");
        assert_eq!(&conjugate("fix", VerbPerson::It, VerbTense::Present), "fixes");
        assert_eq!(&conjugate("buzz", VerbPerson::It, VerbTense::Present), "buzzes");
        assert_eq!(&conjugate("have", VerbPerson::It, VerbTense::Present), "has");
        assert_eq!(&conjugate("catch", VerbPerson::It, VerbTense::Present), "catches");

        assert_eq!(&conjugate("hit", VerbPerson::I, VerbTense::Present), "hit");
        assert_eq!(&conjugate("hit", VerbPerson::We, VerbTense::Present), "hit");
        assert_eq!(&conjugate("hit", VerbPerson::They, VerbTense::Present), "hit");
    }
}
