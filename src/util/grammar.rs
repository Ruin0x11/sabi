use regex::Regex;

#[derive(PartialEq, Eq)]
pub enum VerbPerson {
    I,
    You,
    He,
    She,
    It,
    We,
    They
}

use self::VerbPerson::*;

impl VerbPerson {
    pub fn pronoun(&self) -> &str {
        match *self {
            I    => "I",
            You  => "you",
            He   => "he",
            She  => "she",
            It   => "it",
            We   => "we",
            They => "they",
        }
    }

    pub fn exists(&self) -> &str {
        match *self {
            I    => "am",
            You  => "are",
            He   => "is",
            She  => "is",
            It   => "is",
            We   => "are",
            They => "are",
        }
    }

    pub fn existed(&self) -> &str {
        match *self {
            I    => "was",
            You  => "were",
            He   => "was",
            She  => "was",
            It   => "was",
            We   => "were",
            They => "were",
        }
    }

    pub fn possessive(&self) -> &str {
        match *self {
            I    => "my",
            You  => "your",
            He   => "his",
            She  => "her",
            It   => "its",
            We   => "our",
            They => "their",
        }
    }

    pub fn accusative(&self) -> &str {
        match *self {
            I    => "me",
            You  => "you",
            He   => "him",
            She  => "her",
            It   => "it",
            We   => "us",
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

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

macro_rules! rule {
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
    rule!(name, "(?P<a>[cs])h$", "${a}hes");
    rule!(name, "(?P<a>[cs]h|[zx])$", "${a}es");
    rule!(name, "(?P<a>us)$", "${a}es");
    rule!(name, "ss$", "sses");
    rule!(name, "s$", "ses");
    rule!(name, "$", "s");

    name.to_string()
}

fn is_vowel(ch: char) -> bool {
    match ch {
        'a' |
        'e' |
        'i' |
        'o' |
        'u' => true,
        _   => false,
    }
}

fn get_article(name: &str) -> &str {
    if name.is_empty() {
        return "";
    }

    let first_char = |s: &str| s.chars().next().unwrap();
    let first = first_char(name);

    if !is_vowel(first) {
        "a"
    } else {
        "an"
    }
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
        "I"    => "my".to_string(),
        "you"  => "your".to_string(),
        "it"   => "its".to_string(),
        "he"   => "his".to_string(),
        "she"  => "her".to_string(),
        "we"   => "our".to_string(),
        "they" => "their".to_string(),
        _ => {
            match last_char(s) {
                Some('s') => format!("{}'", s),
                _         => format!("{}'s", s),
            }
        }
    }
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
}
