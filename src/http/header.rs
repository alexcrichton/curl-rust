use std::str;

use self::State::{HdrNameStart, HdrName, HdrValStart, HdrNameDiscardWs, HdrVal,
                  HdrValDiscardWs};

enum State {
    HdrNameStart,
    HdrName,
    HdrNameDiscardWs,
    HdrValStart,
    HdrVal,
    HdrValDiscardWs
}

/**
 * Simple header parser extracts the header name and value, stripping out
 * starting and trailing white space. It does not, however, normalize header
 * value whitespace
 */
pub fn parse<'a>(buf: &'a [u8]) -> Option<(&'a str, &'a str)> {
    let mut i = 0;
    let mut name_begin = 0;
    let mut name_end = 0;
    let mut val_begin = 0;
    let mut val_end = 0;
    let mut state = HdrNameStart;

    while i < buf.len() {
        let c = buf[i];

        match state {
            HdrNameStart => {
                if is_token(c) {
                    name_begin = i;
                    name_end = i;
                    state = HdrName;
                }
                else if c == COLON {
                    name_end = i;
                    state = HdrValStart;
                }
                else if is_space(c) {
                    name_end = i;
                }
                else {
                    return None; // error
                }
            }
            HdrName => {
                if c == COLON {
                    name_end = i;
                    state = HdrValStart;
                }
                else if is_space(c) {
                    name_end = i;
                    state = HdrNameDiscardWs;
                }
                else if !is_token(c) {
                    return None; // error
                }
            }
            HdrNameDiscardWs => {
                if is_token(c) {
                    state = HdrName;
                }
                else if c == COLON {
                    state = HdrValStart;
                }
                else if !is_space(c) {
                    return None; // error
                }
            }
            HdrValStart => {
                if !is_lws(c) {
                    val_begin = i;
                    val_end = i + 1;
                    state = HdrVal;
                }
            }
            HdrVal => {
                if is_lws(c) {
                    val_end = i;
                    state = HdrValDiscardWs;
                }
                else {
                    val_end = i + 1;
                }
            }
            HdrValDiscardWs => {
                if !is_lws(c) {
                    val_end = i + 1;
                    state = HdrVal;
                }
            }
        }

        i += 1;
    }

    let name = match str::from_utf8(&buf[name_begin..name_end]) {
        Ok(v) => v,
        Err(..) => return None
    };

    let val = match str::from_utf8(&buf[val_begin..val_end]) {
        Ok(v) => v,
        Err(..) => return None
    };

    Some((name, val))
}

static COLON: u8 = 58;

static TOKEN: &'static [u8] = &[
/*   0 nul    1 soh    2 stx    3 etx    4 eot    5 enq    6 ack    7 bel  */
        0,       0,       0,       0,       0,       0,       0,       0,
/*   8 bs     9 ht    10 nl    11 vt    12 np    13 cr    14 so    15 si   */
        0,       0,       0,       0,       0,       0,       0,       0,
/*  16 dle   17 dc1   18 dc2   19 dc3   20 dc4   21 nak   22 syn   23 etb */
        0,       0,       0,       0,       0,       0,       0,       0,
/*  24 can   25 em    26 sub   27 esc   28 fs    29 gs    30 rs    31 us  */
        0,       0,       0,       0,       0,       0,       0,       0,
/*  32 sp    33  !    34  "    35  #    36  $    37  %    38  &    39  '  */
        0,      33,       0,      35,      36,      37,      38,      39,
/*  40  (    41  )    42  *    43  +    44  ,    45  -    46  .    47  /  */
        0,       0,      42,      43,      44,      45,      46,      47,
/*  48  0    49  1    50  2    51  3    52  4    53  5    54  6    55  7  */
       48,      49,      50,      51,      52,      53,      54,      55,
/*  56  8    57  9    58  :    59  ;    60  <    61  =    62  >    63  ?  */
       56,      57,       0,       0,       0,       0,       0,       0,
/*  64  @    65  A    66  B    67  C    68  D    69  E    70  F    71  G  */
        0,      97,      98,      99,     100,     101,     102,     103,
/*  72  H    73  I    74  J    75  K    76  L    77  M    78  N    79  O  */
      104,     105,     106,     107,     108,     109,     110,     111,
/*  80  P    81  Q    82  R    83  S    84  T    85  U    86  V    87  W  */
      112,     113,     114,     115,     116,     117,     118,     119,
/*  88  X    89  Y    90  Z    91  [    92  \    93  ]    94  ^    95  _  */
      120,     121,     122,       0,       0,       0,      94,      95,
/*  96  `    97  a    98  b    99  c   100  d   101  e   102  f   103  g  */
       96,      97,      98,      99,     100,     101,     102,     103,
/* 104  h   105  i   106  j   107  k   108  l   109  m   110  n   111  o  */
      104,     105,     106,     107,     108,     109,     110,     111,
/* 112  p   113  q   114  r   115  s   116  t   117  u   118  v   119  w  */
      112,     113,     114,     115,     116,     117,     118,     119,
/* 120  x   121  y   122  z   123  {   124  |   125  }   126  ~   127 del */
      120,     121,     122,     123,     124,       0,     126,       0
];

#[inline]
fn is_token(c: u8) -> bool {
    c < 128 && TOKEN[c as usize] > 0
}

#[inline]
fn is_space(c: u8) -> bool {
    c == (' ' as u8) || c == ('\t' as u8)
}

#[inline]
fn is_lws(c: u8) -> bool {
    is_space(c) || c == ('\n' as u8) || c == ('\r' as u8)
}

#[cfg(test)]
mod test {
    use super::parse;

    fn parse_str<'a>(s: &'a str) -> (&'a str, &'a str) {
        parse(s.as_bytes()).unwrap()
    }

    #[test]
    pub fn test_basic_header() {
        let (name, val) = parse_str("foo: bar");

        assert!(name == "foo");
        assert!(val == "bar");
    }

    #[test]
    pub fn test_basic_header_with_crlf() {
        let (name, val) = parse_str("foo: bar\r\n");

        assert!(name == "foo");
        assert!(val == "bar");
    }

    #[test]
    pub fn test_header_with_extra_spacing() {
        let (name, val) = parse_str(" \tfoo  :bar \t\r");

        assert!(name == "foo");
        assert!(val == "bar");
    }

    #[test]
    pub fn test_header_without_value() {
        let (name, val) = parse_str("foo:");

        assert!(name == "foo");
        assert!(val == "");
    }

    #[test]
    pub fn test_header_value_with_spacing_characters() {
        let (name, val) = parse_str("foo: blah@example.com\r\n");

        assert!(name == "foo");
        assert!(val == "blah@example.com");
    }

    #[test]
    pub fn test_parsing_empty_line() {
        let res = parse("\r\n\r\n".as_bytes());
        assert!(res.is_none());
    }

    #[test]
    pub fn test_parsing_invalid_bytes() {
        let res = parse(b"fo\x9co: zomg");
        assert!(res.is_none());
    }
}
