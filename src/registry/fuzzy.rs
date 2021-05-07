use std::cmp::Ordering;
use std::path::PathBuf;

pub fn eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|cmp| match cmp {
        (b'-', b'_') | (b'_', b'-') => true,
        (a, b) => a == b,
    })
}

pub fn starts_with(a: &str, b: &str) -> bool {
    if b.len() > a.len() {
        return false;
    }
    eq(&a[..b.len()], b)
}

#[allow(clippy::ptr_arg)]
pub fn cmp(a: &PathBuf, b: &PathBuf) -> Ordering {
    let replace = |e: &PathBuf| e.file_name().unwrap().to_str().map(|s| s.replace('_', "-"));
    replace(a).cmp(&replace(b))
}
