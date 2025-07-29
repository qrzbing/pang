//! Symbol contains Terminal or NonTerminal.

use serde::{Deserialize, Serialize};

/// BinaryKind, for binary terminal has three types.
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryKind {
    /// Fixed Bytes.
    Bytes {
        /// Bytes size
        size: usize,
    },
    /// Fixed size bits, no more than 8 Bytes.
    Bits {
        /// Bits size
        size: usize,
    },
    /// Dynamic Bytes size
    Dynamic,
}

/// TerminalKind, for terminal symbols in grammar.
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub enum TerminalKind {
    /// Literal string, most if the time is irrelevant.
    Literal(Vec<u8>),
    /// Binary data, maybe contextually related.
    Binary(BinaryKind),
}

/// Create a NonTerminal.
pub fn nt(label: &str) -> Symbol {
    Symbol::NonTerminal {
        label: label.to_string(),
    }
}

/// Create a Literal Terminal.
pub fn t(value: &[u8]) -> Symbol {
    Symbol::Terminal {
        kind: TerminalKind::Literal(value.to_vec()),
    }
}

/// Create a Binary Bytes Terminal.
pub fn t_bytes(size: usize) -> Symbol {
    Symbol::Terminal {
        kind: TerminalKind::Binary(BinaryKind::Bytes { size }),
    }
}

/// Create a Binary Bits Terminal.
pub fn t_bits(size: usize) -> Symbol {
    Symbol::Terminal {
        kind: TerminalKind::Binary(BinaryKind::Bits { size }),
    }
}

/// Create a Dynamic Binary Terminal.
pub fn t_dyn() -> Symbol {
    Symbol::Terminal {
        kind: TerminalKind::Binary(BinaryKind::Dynamic),
    }
}

/// Symbol contains Terminal or NonTerminal.
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub enum Symbol {
    /// Terminal can not be expanded.
    Terminal {
        /// Terminal points to a TerminalKind.
        kind: TerminalKind,
    },
    /// NonTerminal can be expanded by other symbols.
    NonTerminal {
        /// NonTerminal has a label as its name.
        label: String,
    },
}

impl Symbol {
    /// Display a symbol in a human-readable format.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::grammar::{Symbol, t_bytes, t_dyn, t_bits, t, nt};
    ///
    /// let sym = nt("S");
    /// assert_eq!(sym.display_symbol(), "<S>");
    ///
    /// let sym = t_bytes(10);
    /// assert_eq!(sym.display_symbol(), "Bytes[10]");
    ///
    /// let sym = t_bits(10);
    /// assert_eq!(sym.display_symbol(), "Bits[10]");
    ///
    /// let sym = t_dyn();
    /// assert_eq!(sym.display_symbol(), "Dynamic");
    ///
    /// let sym = t(b"hello");
    /// assert_eq!(sym.display_symbol(), "\"hello\" [68 65 6c 6c 6f]");
    ///
    /// ```
    pub fn display_symbol(&self) -> String {
        use BinaryKind::*;
        use TerminalKind::*;
        match self {
            Symbol::NonTerminal { label } => format!("<{}>", label),
            Symbol::Terminal { kind } => match kind {
                Literal(value) => format!("{}", format_literal_value(value)),
                Binary(bin_kind) => match bin_kind {
                    Bytes { size } => format!("Bytes[{}]", size),
                    Bits { size } => format!("Bits[{}]", size),
                    Dynamic => format!("Dynamic"),
                },
            },
        }
    }

    /// Get the label of a NonTerminal symbol.
    pub fn label(&self) -> &str {
        match self {
            Symbol::NonTerminal { label } => label,
            _ => panic!("Cannot call .label() on a Terminal symbol"),
        }
    }

    /// Get the value of a literal terminal symbol.
    pub fn value(&self) -> &[u8] {
        match self {
            Symbol::Terminal {
                kind: TerminalKind::Literal(value),
            } => value,
            _ => panic!("Cannot call .value() on a NonTerminal or Non-literal symbol"),
        }
    }
}

/// Shows the value of a literal terminal symbol.
fn format_literal_value(value: &[u8]) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }

    let hex_str = value
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");

    match String::from_utf8(value.to_vec()) {
        Ok(utf8_str) => {
            if utf8_str.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
                format!("\"{}\" [{}]", utf8_str, hex_str)
            } else if utf8_str
                .chars()
                .all(|c| !c.is_control() || c == '\n' || c == '\t')
            {
                // Escape non-graphic characters
                let escaped = utf8_str
                    .replace('\n', "\\n")
                    .replace('\t', "\\t")
                    .replace('\r', "\\r");
                format!("\"{}\" [{}]", escaped, hex_str)
            } else {
                format!("[{}]", hex_str)
            }
        }
        Err(_) => {
            format!("[{}]", hex_str)
        }
    }
}
