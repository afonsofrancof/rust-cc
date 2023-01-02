use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Domain {
    parts: Vec<String>,
}

impl Domain {
    pub fn new_empty() -> Domain{
        Domain { parts: Vec::new()}
    }

    pub fn new(input: String) -> Domain {
        // If the input string is empty or consists of only a `"."` character,
        // create a `parts` vector that is empty.
        let parts: Vec<String>;
        if input.is_empty() || input == "." {
            parts = vec![];
        } else {
            // Remove any trailing dots from the input string
            let input = input.trim_end_matches('.');

            // Otherwise, split the input string on the `.` character using
            // `split_terminator()` and store the resulting parts in a vector
            // of strings.
            parts = input.split_terminator('.').map(|s| s.to_string()).collect();
        }

        Domain { parts }
    }

    pub fn is_subdomain_of(&self, other: &Domain) -> bool {
        // If `other` is the root domain, then `self` is always considered
        // to be a subdomain of `other`, regardless of the number of parts
        // in `self` or the values of those parts.
        if other.is_root() {
            return true;
        }

        // Check if the number of parts in `self` is greater than or equal to
        // the number of parts in `other`.
        if self.parts.len() < other.parts.len() {
            return false;
        }

        // Check if each corresponding pair of parts in the two domains
        // are equal starting from the end of both vectors.
        self.parts
            .iter()
            .rev()
            .zip(other.parts.iter().rev())
            .all(|(a, b)| a == b)
    }

    pub fn is_root(&self) -> bool {
        // Check if the domain is the root by checking if it has no parts.
        self.parts.is_empty()
    }

    pub fn to_string(&self) -> String {
        if self.is_root() {
            // If the domain is the root, return a single dot.
            ".".to_string()
        } else {
            // Otherwise, join the parts of the domain with `.` characters and
            // return the resulting string.
            let mut st = self.parts.join(".");
            st.push('.');
            st
        }
    }


    pub fn getparts(&self) -> &Vec<String> {
        &self.parts
    }

    pub fn set_parts(&mut self, new_parts: Vec<String>) {
        self.parts = new_parts;
    }
}

#[test]
fn test_new_with_empty_string() {
    let domain = Domain::new("".to_string());
    assert!(domain.is_root());
}

#[test]
fn test_new_with_dot() {
    let domain = Domain::new(".".to_string());
    assert!(domain.is_root());
}

#[test]
fn test_new_with_domain() {
    let domain = Domain::new("example.com".to_string());
    assert_eq!(domain.getparts(), &vec!["example".to_string(), "com".to_string()]);
}

#[test]
fn test_trailing_dot_domain(){
    let domain = Domain::new("example.com".to_string());
    let domain2 = Domain::new("example.com.".to_string());
    assert_eq!(domain.to_string(),domain2.to_string());
}

#[test]
fn test_is_subdomain_of() {
    let example_com = Domain::new("example.com".to_string());
    let foo_example_com = Domain::new("foo.example.com".to_string());
    let bar_foo_example_com = Domain::new("bar.foo.example.com".to_string());
    let root_with_dot = Domain::new(".".to_string());
    let root_without_dot = Domain::new("".to_string());

    assert!(foo_example_com.is_subdomain_of(&example_com));
    assert!(!example_com.is_subdomain_of(&foo_example_com));
    assert!(bar_foo_example_com.is_subdomain_of(&foo_example_com));
    assert!(!foo_example_com.is_subdomain_of(&bar_foo_example_com));
    assert!(!root_with_dot.is_subdomain_of(&foo_example_com));
    assert!(!root_without_dot.is_subdomain_of(&foo_example_com));
}

#[test]
fn test_is_root() {
    let domain = Domain::new("".to_string());
    assert!(domain.is_root());

    let domain = Domain::new(".".to_string());
    assert!(domain.is_root());

    let domain = Domain::new("example.com".to_string());
    assert!(!domain.is_root());
}

#[test]
fn test_to_string() {
    let mut domain = Domain::new("".to_string());
    assert_eq!(domain.to_string(), ".");

    domain.set_parts(vec!["example".to_string(), "com".to_string()]);
    assert_eq!(domain.to_string(), "example.com");

    domain.set_parts(vec![]);
    assert_eq!(domain.to_string(), ".");
}
