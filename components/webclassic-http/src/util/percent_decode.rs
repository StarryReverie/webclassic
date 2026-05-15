fn decode_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

pub fn decode_percent(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());

    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hi = decode_hex(bytes[i + 1]);
                let lo = decode_hex(bytes[i + 2]);
                match (hi, lo) {
                    (Some(h), Some(l)) => {
                        result.push(h << 4 | l);
                        i += 3;
                    }
                    _ => {
                        result.push(b'%');
                        i += 1;
                    }
                }
            }
            b'+' => {
                result.push(b' ');
                i += 1;
            }
            b => {
                result.push(b);
                i += 1;
            }
        }
    }

    String::from_utf8_lossy(&result).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_encoding() {
        assert_eq!(decode_percent("hello"), "hello");
    }

    #[test]
    fn percent_hex() {
        assert_eq!(decode_percent("hello%20world"), "hello world");
    }

    #[test]
    fn plus_to_space() {
        assert_eq!(decode_percent("hello+world"), "hello world");
    }

    #[test]
    fn mixed() {
        assert_eq!(
            decode_percent("a=1%262&b=hello+world"),
            "a=1&2&b=hello world"
        );
    }

    #[test]
    fn incomplete_percent() {
        assert_eq!(decode_percent("100%"), "100%");
        assert_eq!(decode_percent("100%2"), "100%2");
    }

    #[test]
    fn invalid_hex() {
        assert_eq!(decode_percent("%ZZ"), "%ZZ");
    }

    #[test]
    fn multibyte_utf8() {
        assert_eq!(decode_percent("%E4%BD%A0%E5%A5%BD"), "你好");
    }
}
