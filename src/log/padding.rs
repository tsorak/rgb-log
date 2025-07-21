use std::collections::HashMap;

pub struct PadLeft<'a> {
    pub width: u8,
    pub map: HashMap<&'a str, u8>,
}

impl<'a> PadLeft<'a> {
    pub fn new<T>(iterable: T) -> Self
    where
        T: IntoIterator<Item = &'a str>,
    {
        let (width, map) = to_pad_map_left(iterable);
        Self { width, map }
    }

    pub fn get(&self, k: &'a str) -> String {
        let (padding, key) = self.get_split(k);
        format!("{padding}{key}")
    }

    /// Get per key. If None, compute padding per widest known key
    pub fn get_split(&self, k: &'a str) -> (String, &'a str) {
        match self.map.get(k) {
            Some(n) => {
                let padding = " ".repeat((*n).into());
                (padding, k)
            }
            None => {
                let letters_count = k.chars().count() as u8;

                if letters_count < self.width {
                    let n = self.width - letters_count;
                    let s = " ".repeat(n.into());
                    (s, k)
                } else {
                    (String::new(), k)
                }
            }
        }
    }
}

/// v is k with left padding.
/// Padding amount is the number of letters in the longest k letter-wise.
fn to_pad_map_left<'a, T>(iterable: T) -> (u8, HashMap<&'a str, u8>)
where
    T: IntoIterator<Item = &'a str>,
{
    let map: HashMap<&'a str, u8> = iterable
        .into_iter()
        .map(|s| {
            let letters_count = s.chars().count() as u8;

            (s, letters_count)
        })
        .collect();

    let mut widest = 0;
    for letters_count in map.values() {
        if widest < *letters_count {
            widest = *letters_count;
        }
    }

    // Set the padding that each one needs
    let pad_map = map
        .into_iter()
        .map(|(k, v)| {
            let needed_padding = widest - v;
            (k, needed_padding)
        })
        .collect();

    (widest, pad_map)
}

#[test]
fn default_to_pad_map_left() {
    const DEFAULT_LEVELS: [&str; 2] = ["INFO", "ERROR"];

    let mut v = to_pad_map_left(DEFAULT_LEVELS)
        .1
        .into_iter()
        .collect::<Vec<(&str, u8)>>();
    v.sort_by(|(a, _), (b, _)| a.cmp(b));

    let expected = vec![("ERROR", 0), ("INFO", 1)];

    assert_eq!(v, expected);
}
