use std::{convert::TryFrom, fmt};

#[cfg(target_arch = "wasm32")]
use serde::Serialize;

#[cfg(not(feature = "std"))]
use alloc::{
  string::{String, ToString},
  vec::Vec,
};

/// Token which represents a valids CDDL character or sequence
#[derive(PartialEq, Debug)]
pub enum Token {
  /// Illegal sequence of characters
  ILLEGAL(String),
  /// End of file
  EOF,

  /// Identifier with optional `SocketPlug`
  IDENT((String, Option<SocketPlug>)),
  /// Value
  VALUE(Value),
  /// CBOR tag '#'
  TAG((Option<u8>, Option<usize>)),

  // Operators
  /// Assignment operator '='
  ASSIGN,
  /// Optional occurrence indicator '?'
  OPTIONAL,
  /// Zero or more occurrence indicator '*'
  ASTERISK,
  /// One or more occurrence indicator '+'
  ONEORMORE,
  /// Unwrap operator '~'
  UNWRAP,

  // Delimiters
  /// Comma ','
  COMMA,
  /// Colon ':'
  COLON,

  /// Comment text
  COMMENT(String),

  /// Type choice indicator '/'
  TCHOICE,
  /// Group choice indicator '//'
  GCHOICE,
  /// Type choice alternative '/='
  TCHOICEALT,
  /// Group choice alternative '//='
  GCHOICEALT,
  /// Arrow map '=>'
  ARROWMAP,
  /// Cut '^'
  CUT,

  /// Range operator. Inclusive '..' if true, otherwise exclusive '...'s
  RANGEOP(bool),

  /// Range tuple with lower bound, upper bound, and bool indicating whether or
  /// not the range is inclusive
  RANGE((RangeValue, RangeValue, bool)),

  /// Left opening parend
  LPAREN,
  /// Right closing parend
  RPAREN,
  /// Left opening brace
  LBRACE,
  /// Right closing brace
  RBRACE,
  /// Left opening bracket
  LBRACKET,
  /// Right closing bracket
  RBRACKET,
  /// Left opening angle bracket
  LANGLEBRACKET,
  /// Right closing angle bracket
  RANGLEBRACKET,

  // Control operators
  /// .size control operator
  SIZE,
  /// .bits control operator
  BITS,
  /// .regexp control operator
  CREGEXP,
  /// .cbor control operator
  CBOR,
  /// .cborseq control operator
  CBORSEQ,
  /// .within control operator
  WITHIN,
  /// .and control operator
  AND,
  /// .lt control operator
  LT,
  /// .le control operator
  LE,
  /// .gt control operator
  GT,
  /// .ge control operator
  GE,
  /// .eq control operator
  EQ,
  /// .ne control operator
  NE,
  /// .default control operator
  DEFAULT,
  /// .pcre control operator
  /// Proposed control extension to support Perl-Compatible Regular Expressions
  /// (PCREs). See https://tools.ietf.org/html/rfc8610#section-3.8.3.2s
  PCRE,

  /// group to choice enumeration '&'
  GTOCHOICE,

  // Standard prelude
  /// false
  FALSE,
  /// true
  TRUE,
  /// bool
  BOOL,
  /// nil
  NIL,
  /// null
  NULL,
  /// uint
  UINT,
  /// nint
  NINT,
  /// int
  INT,
  /// float16
  FLOAT16,
  /// float32
  FLOAT32,
  /// float64
  FLOAT64,
  /// float16-32
  FLOAT1632,
  /// float32-64
  FLOAT3264,
  ///float
  FLOAT,
  /// bstr
  BSTR,
  /// tstr
  TSTR,
  /// any
  ANY,
  /// bytes
  BYTES,
  /// text
  TEXT,
  /// tdate
  TDATE,
  /// time
  TIME,
  /// number
  NUMBER,
  /// biguint
  BIGUINT,
  /// bignint
  BIGNINT,
  /// bigint
  BIGINT,
  /// integer
  INTEGER,
  /// unsigned
  UNSIGNED,
  /// decfrac
  DECFRAC,
  /// bigfloat
  BIGFLOAT,
  /// eb64url
  EB64URL,
  /// eb64legacy
  EB64LEGACY,
  /// eb16k
  EB16,
  /// encoded-cbor
  ENCODEDCBOR,
  /// uri
  URI,
  /// b64url
  B64URL,
  /// b64legacy
  B64LEGACY,
  /// regexp
  REGEXP,
  /// mime-message
  MIMEMESSAGE,
  /// cbor-any
  CBORANY,
  /// undefined
  UNDEFINED,
}

impl Token {
  /// Returns optional string literal of token in standard prelude
  pub fn in_standard_prelude(&self) -> Option<&'static str> {
    match self {
      Token::ANY => Some("any"),
      Token::UINT => Some("uint"),
      Token::NINT => Some("nint"),
      Token::INT => Some("int"),
      Token::BSTR => Some("bstr"),
      Token::BYTES => Some("bytes"),
      Token::TSTR => Some("tstr"),
      Token::TEXT => Some("text"),
      Token::TDATE => Some("tdate"),
      Token::TIME => Some("time"),
      Token::NUMBER => Some("number"),
      Token::BIGUINT => Some("biguint"),
      Token::BIGNINT => Some("bignint"),
      Token::BIGINT => Some("bigint"),
      Token::INTEGER => Some("integer"),
      Token::UNSIGNED => Some("unsigned"),
      Token::DECFRAC => Some("decfrac"),
      Token::BIGFLOAT => Some("bigfloat"),
      Token::EB64URL => Some("eb64url"),
      Token::EB64LEGACY => Some("eb64legacy"),
      Token::EB16 => Some("eb16"),
      Token::ENCODEDCBOR => Some("encoded-cbor"),
      Token::URI => Some("uri"),
      Token::B64URL => Some("b64url"),
      Token::B64LEGACY => Some("b64legacy"),
      Token::REGEXP => Some("regexp"),
      Token::MIMEMESSAGE => Some("mime-message"),
      Token::CBORANY => Some("cbor-any"),
      Token::FLOAT16 => Some("float16"),
      Token::FLOAT32 => Some("float32"),
      Token::FLOAT64 => Some("float64"),
      Token::FLOAT1632 => Some("float16-32"),
      Token::FLOAT3264 => Some("float32-64"),
      Token::FLOAT => Some("float"),
      Token::FALSE => Some("false"),
      Token::TRUE => Some("true"),
      Token::BOOL => Some("bool"),
      Token::NIL => Some("nil"),
      Token::NULL => Some("null"),
      Token::UNDEFINED => Some("undefined"),
      _ => None,
    }
  }
}

/// Range value
#[derive(Debug, PartialEq, Clone)]
pub enum RangeValue {
  /// Identifier with optional socket/plug prefix
  IDENT((String, Option<SocketPlug>)),
  /// Integer
  INT(isize),
  /// Unsigned integer
  UINT(usize),
  /// Float
  FLOAT(f64),
}

impl TryFrom<Token> for RangeValue {
  type Error = &'static str;

  fn try_from(t: Token) -> Result<Self, Self::Error> {
    match t {
      Token::IDENT(ident) => Ok(RangeValue::IDENT(ident)),
      Token::VALUE(value) => match value {
        Value::INT(i) => Ok(RangeValue::INT(i)),
        Value::UINT(ui) => Ok(RangeValue::UINT(ui)),
        Value::FLOAT(f) => Ok(RangeValue::FLOAT(f)),
        _ => Err("Invalid range token"),
      },
      _ => Err("Invalid range token"),
    }
  }
}

impl RangeValue {
  /// Returns `Value` from given `RangeValue`
  pub fn as_value(&self) -> Option<Value> {
    match &self {
      RangeValue::UINT(ui) => Some(Value::UINT(*ui)),
      RangeValue::FLOAT(f) => Some(Value::FLOAT(*f)),
      _ => None,
    }
  }
}

impl fmt::Display for RangeValue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      RangeValue::IDENT(ident) => write!(f, "{}", ident.0),
      RangeValue::INT(i) => write!(f, "{}", i),
      RangeValue::UINT(i) => write!(f, "{}", i),
      RangeValue::FLOAT(fl) => write!(f, "{}", fl),
    }
  }
}

/// Literal string value
// TODO: support hexfloat and exponent
#[cfg_attr(target_arch = "wasm32", derive(Serialize))]
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
  /// Integer value
  INT(isize),
  /// Unsigned integer value
  UINT(usize),
  /// Float value
  FLOAT(f64),
  /// Text value
  TEXT(String),
  /// Byte value
  BYTE(ByteValue),
}

/// Numeric value
#[derive(Debug, PartialEq)]
pub enum Numeric {
  /// Integer
  INT(isize),
  /// Unsigned integer
  UINT(usize),
  /// Float
  FLOAT(f64),
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Value::TEXT(text) => write!(f, "\"{}\"", text),
      Value::INT(i) => write!(f, "{}", i),
      Value::UINT(ui) => write!(f, "{}", ui),
      Value::FLOAT(float) => write!(f, "{}", float),
      Value::BYTE(bv) => write!(f, "{}", bv),
    }
  }
}

impl<'a> From<&'static str> for Value {
  fn from(value: &'static str) -> Self {
    Value::TEXT(value.into())
  }
}

/// Byte string values
#[cfg_attr(target_arch = "wasm32", derive(Serialize))]
#[derive(Debug, PartialEq, Clone)]
pub enum ByteValue {
  /// Unprefixed byte string value
  UTF8(Vec<u8>),
  /// Prefixed base16 encoded byte string value
  B16(Vec<u8>),
  /// Prefixed base64 encoded (URL safe) byte string value
  B64(Vec<u8>),
}

impl fmt::Display for ByteValue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ByteValue::UTF8(b) => write!(f, "'{}'", std::str::from_utf8(b).map_err(|_| fmt::Error)?),
      ByteValue::B16(b) => write!(
        f,
        "h'{}'",
        String::from_utf8(b.to_vec())
          .map_err(|_| fmt::Error)?
          .replace(" ", "")
      ),
      ByteValue::B64(b) => write!(
        f,
        "b64'{}'",
        String::from_utf8(b.to_vec())
          .map_err(|_| fmt::Error)?
          .replace(" ", "")
      ),
    }
  }
}

/// Socket/plug prefix
#[cfg_attr(target_arch = "wasm32", derive(Serialize))]
#[derive(Debug, PartialEq, Clone)]
pub enum SocketPlug {
  /// Type socket `$`
  TYPE,
  /// Group socket `$$`
  GROUP,
}

impl std::str::FromStr for SocketPlug {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if let Some(c) = s.chars().next() {
      if c == '$' {
        if let Some(c) = s.chars().nth(1) {
          if c == '$' {
            return Ok(SocketPlug::GROUP);
          }
        }

        return Ok(SocketPlug::TYPE);
      }
    }

    Err("Malformed socket plug string")
  }
}

impl<'a> fmt::Display for SocketPlug {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      SocketPlug::TYPE => write!(f, "$"),
      SocketPlug::GROUP => write!(f, "$$"),
    }
  }
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Token::IDENT((ident, socket_plug)) => {
        if let Some(sp) = socket_plug {
          return write!(f, "{}{}", sp, ident);
        }

        write!(f, "{}", ident)
      }
      Token::ILLEGAL(s) => write!(f, "ILLEGAL({})", s),
      Token::ASSIGN => write!(f, "="),
      Token::ONEORMORE => write!(f, "+"),
      Token::OPTIONAL => write!(f, "?"),
      Token::ASTERISK => write!(f, "*"),
      Token::LPAREN => write!(f, "("),
      Token::RPAREN => write!(f, ")"),
      Token::LBRACE => write!(f, "{{"),
      Token::RBRACE => write!(f, "}}"),
      Token::LBRACKET => write!(f, "["),
      Token::RBRACKET => write!(f, "]"),
      Token::TCHOICE => write!(f, "/"),
      Token::TCHOICEALT => write!(f, "/="),
      Token::GCHOICEALT => write!(f, "//="),
      Token::COMMA => write!(f, ","),
      Token::COMMENT(c) => write!(f, ";{}", c),
      Token::COLON => write!(f, ":"),
      Token::CUT => write!(f, "^"),
      Token::EOF => write!(f, ""),
      Token::TSTR => write!(f, "tstr"),
      Token::LANGLEBRACKET => write!(f, "<"),
      Token::RANGLEBRACKET => write!(f, ">"),
      Token::INT => write!(f, "int"),
      Token::UINT => write!(f, "uint"),
      Token::ARROWMAP => write!(f, "=>"),
      Token::SIZE => write!(f, ".size"),
      Token::BITS => write!(f, ".bits"),
      Token::REGEXP => write!(f, ".regexp"),
      Token::PCRE => write!(f, ".pcre"),
      Token::CBOR => write!(f, ".cbor"),
      Token::CBORSEQ => write!(f, ".cborseq"),
      Token::WITHIN => write!(f, ".within"),
      Token::AND => write!(f, ".and"),
      Token::LT => write!(f, ".lt"),
      Token::LE => write!(f, ".le"),
      Token::GT => write!(f, ".gt"),
      Token::GE => write!(f, ".ge"),
      Token::EQ => write!(f, ".eq"),
      Token::NE => write!(f, ".ne"),
      Token::DEFAULT => write!(f, ".default"),
      Token::NUMBER => write!(f, "number"),
      Token::BSTR => write!(f, "bstr"),
      Token::BYTES => write!(f, "bytes"),
      Token::GCHOICE => write!(f, "//"),
      Token::TRUE => write!(f, "true"),
      Token::GTOCHOICE => write!(f, "&"),
      Token::VALUE(value) => write!(f, "{}", value),
      Token::RANGEOP(i) => {
        if *i {
          write!(f, "..")
        } else {
          write!(f, "...")
        }
      }
      Token::RANGE((l, u, i)) => match l {
        RangeValue::IDENT(_) if *i => write!(f, "{} .. {}", l, u),
        RangeValue::IDENT(_) => write!(f, "{} ... {}", l, u),
        _ => {
          if *i {
            write!(f, "{}..{}", l, u)
          } else {
            write!(f, "{}...{}", l, u)
          }
        }
      },
      Token::TAG((mt, tag)) => {
        if let Some(m) = mt {
          if let Some(t) = tag {
            return write!(f, "#{}.{}", m, t);
          }

          return write!(f, "#{}", m);
        }
        write!(f, "#")
      }
      _ => write!(f, ""),
    }
  }
}

/// Return an optional control token from a given string
pub fn lookup_control_from_str(ident: &str) -> Option<Token> {
  match ident {
    ".size" => Some(Token::SIZE),
    ".bits" => Some(Token::BITS),
    ".regexp" => Some(Token::CREGEXP),
    ".cbor" => Some(Token::CBOR),
    ".cborseq" => Some(Token::CBORSEQ),
    ".within" => Some(Token::WITHIN),
    ".and" => Some(Token::AND),
    ".lt" => Some(Token::LT),
    ".le" => Some(Token::LE),
    ".gt" => Some(Token::GT),
    ".ge" => Some(Token::GE),
    ".eq" => Some(Token::EQ),
    ".ne" => Some(Token::NE),
    ".default" => Some(Token::DEFAULT),
    ".pcre" => Some(Token::PCRE),
    _ => None,
  }
}

/// Return an optional string from a given token
pub fn control_str_from_token(t: &Token) -> Option<&'static str> {
  match t {
    Token::SIZE => Some(".size"),
    Token::BITS => Some(".bits"),
    Token::REGEXP => Some(".regexp"),
    Token::CBOR => Some(".cbor"),
    Token::CBORSEQ => Some(".cborseq"),
    Token::WITHIN => Some(".within"),
    Token::AND => Some(".and"),
    Token::LT => Some(".lt"),
    Token::LE => Some(".le"),
    Token::GT => Some(".gt"),
    Token::GE => Some(".ge"),
    Token::EQ => Some(".eq"),
    Token::NE => Some(".ne"),
    Token::DEFAULT => Some(".default"),
    Token::PCRE => Some(".pcre"),
    _ => None,
  }
}

/// Returns token in standard prelude from given string
pub fn lookup_ident(ident: &str) -> Token {
  match ident {
    "false" => Token::FALSE,
    "true" => Token::TRUE,
    "bool" => Token::BOOL,
    "nil" => Token::NIL,
    "null" => Token::NULL,
    "uint" => Token::UINT,
    "nint" => Token::NINT,
    "int" => Token::INT,
    "float16" => Token::FLOAT16,
    "float32" => Token::FLOAT32,
    "float64" => Token::FLOAT64,
    "float16-32" => Token::FLOAT1632,
    "float32-64" => Token::FLOAT3264,
    "float" => Token::FLOAT,
    "bstr" => Token::BSTR,
    "tstr" => Token::TSTR,
    "any" => Token::ANY,
    "bytes" => Token::BYTES,
    "text" => Token::TEXT,
    "tdate" => Token::TDATE,
    "time" => Token::TIME,
    "number" => Token::NUMBER,
    "biguint" => Token::BIGUINT,
    "bignint" => Token::BIGNINT,
    "integer" => Token::INTEGER,
    "unsigned" => Token::UNSIGNED,
    "decfrac" => Token::DECFRAC,
    "bigfloat" => Token::BIGFLOAT,
    "eb64url" => Token::EB64URL,
    "eb64legacy" => Token::EB64LEGACY,
    "eb16" => Token::EB16,
    "encoded-cbor" => Token::ENCODEDCBOR,
    "uri" => Token::URI,
    "b64url" => Token::B64URL,
    "b64legacy" => Token::B64LEGACY,
    "regexp" => Token::REGEXP,
    "mime-message" => Token::MIMEMESSAGE,
    "cbor-any" => Token::CBORANY,
    "undefined" => Token::UNDEFINED,
    _ => {
      if let Some(c) = ident.chars().next() {
        if c == '$' {
          if let Some(c) = ident.chars().nth(1) {
            if c == '$' {
              return Token::IDENT((ident[2..].to_string(), Some(SocketPlug::GROUP)));
            }
          }

          return Token::IDENT((ident[1..].to_string(), Some(SocketPlug::TYPE)));
        }
      }

      Token::IDENT((ident.to_string(), None))
    }
  }
}
