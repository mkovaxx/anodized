use anodized::spec;

#[spec(
    requires: input.is_palindrome(),
    ensures: |(prefix, middle)| {
        let reassembled = prefix.chars().chain(middle).chain(prefix.chars().rev());
        reassembled.eq(input.chars())
    },
)]
pub fn halve_palindrome(input: &str) -> (&str, Option<char>) {
    let n = input.chars().count();
    let half = n / 2;

    // Byte offset where the first `half` of chars end.
    let half_end = input
        .char_indices()
        .nth(half)
        .map(|(i, _)| i)
        .unwrap_or(input.len());

    let prefix = &input[..half_end];

    let middle = if n % 2 == 1 {
        input.chars().nth(half)
    } else {
        None
    };

    (prefix, middle)
}

trait Palindromeness: AsRef<str> {
    fn is_palindrome(&self) -> bool {
        self.as_ref().chars().eq(self.as_ref().chars().rev())
    }
}

impl Palindromeness for str {}
