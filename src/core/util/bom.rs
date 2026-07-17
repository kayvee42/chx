/// Strip a UTF-8 BOM from the start of a byte slice.
pub fn strip_bom(bytes: &[u8]) -> &[u8] {
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        &bytes[3..]
    } else {
        bytes
    }
}

/// Strip a UTF-8 BOM from the start of a string slice.
pub fn strip_bom_str(s: &str) -> &str {
    if s.as_bytes().len() >= 3
        && s.as_bytes()[0] == 0xEF
        && s.as_bytes()[1] == 0xBB
        && s.as_bytes()[2] == 0xBF
    {
        &s[3..]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_bom_with_bom() {
        let bytes = [0xEF, 0xBB, 0xBF, b'h', b'e', b'l', b'l', b'o'];
        assert_eq!(strip_bom(&bytes), b"hello");
    }

    #[test]
    fn test_strip_bom_without_bom() {
        let bytes = b"hello";
        assert_eq!(strip_bom(bytes), b"hello");
    }

    #[test]
    fn test_strip_bom_str() {
        let s = "\u{FEFF}hello";
        assert_eq!(strip_bom_str(s), "hello");
    }

    #[test]
    fn test_strip_bom_short() {
        let bytes = [0xEF, 0xBB];
        assert_eq!(strip_bom(&bytes), &[0xEF, 0xBB]);
    }
}
