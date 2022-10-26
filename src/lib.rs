//! # Description
//!
//! // TODO
//!
//! # Quickstart
//!
//! Defining some of rust's escape sequences:
//!
//! ```
//! use char_escape::{Escaper, Rule};
//!
//! const ESCAPER: Escaper<'static> = char_escape::escaper! {
//!     escape_char = '\\',
//!     rules = [
//!         '\n' => 'n',
//!         '\r' => 'r',
//!         '\t' => 't',
//!         '\\' => '\\',
//!         '\'' => '\'',
//!         '"' => '"',
//!     ],
//! };
//!
//! let unescaped = "\n\r\t\\\'\"";
//! let escaped = r#"\n\r\t\\\'\""#;
//!
//! assert_eq!(ESCAPER.escape(unescaped), escaped);
//! assert_eq!(ESCAPER.unescape(escaped).expect("all escape sequences are valid"), unescaped);
//! ```
//!
//! ```
//! use char_escape::{Escaper, Rule};
//!
//! let escaper = char_escape::escaper! {
//!     escape_char = '\\',
//!     rules = ['\n' => 'n', ' ' => 'w'],
//! };
//!
//! let unescaped = "\
//! line1
//! line2
//!
//! line3 with whitespace";
//!
//! let escaped = r"line1\nline2\n\nline3\wwith\wwhitespace";
//!
//! assert_eq!(escaper.escape(unescaped), escaped);
//! ```

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![allow(clippy::must_use_candidate)]

use {
    core::fmt::{self, Display},
    std::error::Error,
};

/// The quick and easy way to create an [`Escaper`].
///
/// Creates a new `const` [`Escaper`] from the rules provided to the macro. Since
/// this produces a `const` it cannot be used to dynamically create [`Escaper`]s.
///
/// If no escape character is specified, it defaults to `'\\'`.
///
/// The macro implicity adds a rule that the escape character is to be escaped as two consecutive
/// escape characters. E.g. if the escape character is `'\\'`, then the macro adds the rule
/// `'\\' => '\\'`. This behaviour can be overridden by explicity adding a rule for escaping the
/// escape character.
///
/// ```
/// # use char_escape::{Rule, Escaper, escaper};
/// #
/// // Note that the escape character doesn't need to be specified, it defaults to '\\'.
/// let escaper = escaper! {
///     '\n' => 'n',
///     '\r' => 'r',
///     '\t' => 't',
/// //  '\\' => '\\', (implicity added)
/// };
///
/// let a_lot_of_boilerplate = Escaper::new('\\', &[
///     Rule {
///         unescaped: '\n',
///         escaped: 'n',
///     },
///     Rule {
///         unescaped: '\r',
///         escaped: 'r',
///     },
///     Rule {
///         unescaped: '\t',
///         escaped: 't',
///     },
///     Rule {
///         unescaped: '\\',
///         escaped: '\\',
///     },
/// ]).expect("rules are valid");
///
/// assert_eq!(escaper, a_lot_of_boilerplate);
/// ```
///
/// Specifying a custom escape character:
/// ```
/// # use char_escape::{Escaper, Rule, escaper};
/// #
/// let escaper = escaper! {
///     escape_char = '#',
///     rules = [
///         '\n' => 'n',
///         '\t' => 't',
/// //      '#' => '#', (implicitly added)
///     ],
/// };
/// #
/// # let reference = Escaper::new('#', &[
/// #     Rule {
/// #         unescaped: '\n',
/// #         escaped: 'n',
/// #     },
/// #     Rule {
/// #         unescaped: '\t',
/// #         escaped: 't',
/// #     },
/// #     Rule {
/// #         unescaped: '#',
/// #         escaped: '#',
/// #     },
/// # ]).expect("rules are valid");
/// #
/// # assert_eq!(escaper, reference);
/// ```
///
/// The following are equivalent:
/// ```
/// # use char_escape::escaper;
/// #
/// let escaper1 = escaper! {
///     '\n' => 'n',
///     '\r' => 'r',
/// };
///
/// let escaper2 = escaper!('\\', ['\n' => 'n', '\r' => 'r']);
///
/// let escaper3 = escaper! {
///     escape_char = '\\',
///     rules = [
///         '\n' => 'n',
///         '\r' => 'r',
///     ],
/// };
///
/// assert_eq!(escaper1, escaper2);
/// assert_eq!(escaper2, escaper3);
/// ```
#[macro_export]
macro_rules! escaper {
    ($(escape_char =)? $escape_char:literal, $(rules =)? [$($unescaped:literal => $escaped:literal),+ $(,)?] $(,)?) => {
        {
            const escape_char: ::std::primitive::char = $escape_char;
            const NUM_RULES: ::std::primitive::usize = 1 + $crate::count_rules!($($unescaped => $escaped),+ ,);

            const RULES: [$crate::Rule; NUM_RULES] = [
                $(
                    {
                        const unescaped: ::std::primitive::char = $unescaped;
                        const escaped: ::std::primitive::char = $escaped;

                        $crate::Rule {
                            unescaped,
                            escaped,
                        }
                    },
                )+
                $crate::Rule {
                    unescaped: escape_char,
                    escaped: escape_char,
                },
            ];

            $crate::Escaper::new_unchecked(
                escape_char,
                &RULES,
            )
        }
    };
    ($($unescaped:literal => $escaped:literal),+ $(,)?) => {
        $crate::escaper!{
            escape_char = '\\',
            rules = [$($unescaped => $escaped),+],
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! count_rules {
    (,) => {0usize};
    ($_unescaped:literal => $_escaped:literal $(, $unescaped:literal => $escaped:literal)* ,) => {
        1usize + $crate::count_rules!($($unescaped => $escaped),* ,)
    };
}

/// Defines how one specific [`char`] should be escaped.
///
/// Escaping `unescaped` will yield `escaped` and unescaping `escaped` will yield `unescaped`.
///
/// See [`Escaper`] for more information.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rule {
    #[allow(missing_docs)]
    pub unescaped: char,
    #[allow(missing_docs)]
    pub escaped: char,
}

/// Escape and unescape strings.
///
/// See the [crate-level documentation](crate) for more detailed information.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Escaper<'a> {
    escape_char: char,
    rules: &'a [Rule],
}

impl<'a> Escaper<'a> {
    /// Create a new [`Escaper`] without verifying that the `rules` are valid.
    ///
    /// It is discouraged to use this function. Use the [`escaper!`] macro or [`Escaper::new()`]
    /// instead.
    ///
    /// If the `rules` don't contain a [`Rule`] for escaping the escape character, the
    /// [`escape()`](Self::escape) and [`unescape()`](Self::unescape) methods will behave
    /// incorrectly.
    pub const fn new_unchecked(escape_char: char, rules: &'a [Rule]) -> Self {
        Self { escape_char, rules }
    }

    /// Create a new [`Escaper`].
    ///
    /// If you don't need to dynamically create an [`Escaper`], use the [`escaper!`] macro.
    ///
    /// # Errors
    ///
    /// If the provided `rules` don't contain a [`Rule`] for escaping the escape character.
    ///
    /// ```
    /// # use char_escape::{Escaper, MissingEscapeCharRule, Rule};
    /// #
    /// let result = Escaper::new('\\', &[
    ///     Rule {
    ///         unescaped: '\n',
    ///         escaped: 'n',
    ///     },
    ///     Rule {
    ///         unescaped: '\t',
    ///         escaped: 't',
    ///     },
    /// ]);
    ///
    /// assert_eq!(result, Err(MissingEscapeCharRule::new()));
    /// ```
    pub fn new(escape_char: char, rules: &'a [Rule]) -> Result<Self, MissingEscapeCharRule> {
        if contains_escape_char_rule(escape_char, rules) {
            Ok(Self { escape_char, rules })
        } else {
            Err(MissingEscapeCharRule::new())
        }
    }

    /// Returns a new [`String`] with the [`char`]s escaped according to the specified rules.
    ///
    /// ```
    /// # use char_escape::escaper;
    /// #
    /// let escaper = escaper! {
    ///     escape_char = '%',
    ///     rules = [
    ///         'a' => 'a',
    ///         'c' => 'c',
    ///         'e' => 'e',
    ///         'p' => 'p',
    ///         's' => 's',
    ///     ],
    /// };
    ///
    /// let unescaped = "escaper.escape(\"escaper\")";
    /// let escaped = "%e%s%c%a%p%er.%e%s%c%a%p%e(\"%e%s%c%a%p%er\")";
    ///
    /// assert_eq!(escaper.escape(unescaped), escaped);
    /// ```
    pub fn escape(&self, s: &str) -> String {
        let mut ret = String::with_capacity(2 * s.len());

        for c in s.chars() {
            match self.escape_char(c) {
                Some(escaped) => {
                    ret.push(self.escape_char);
                    ret.push(escaped);
                }
                None => ret.push(c),
            }
        }

        debug_assert!(self.is_escaped(&ret));

        ret
    }

    fn escape_char(&self, c: char) -> Option<char> {
        Some(self.rules.iter().find(|rule| rule.unescaped == c)?.escaped)
    }

    /// Reverts what [`escape()`](Self::escape) does.
    ///
    // TODO proptest
    /// Escaping a string and then unescaping it is guaranteed to always result in the original
    /// string.
    ///
    /// ```
    /// # use char_escape::escaper;
    /// #
    /// let escaper = escaper! {
    ///     ' ' => 'w',
    /// };
    ///
    /// let unescaped = escaper.unescape(r"S\wP\wA\wC\wE").expect("is unescapable");
    ///
    /// assert_eq!(unescaped, "S P A C E");
    /// ```
    ///
    /// # Errors
    ///
    /// Fails if the string to escape contains an invalid escape sequence ...
    ///
    /// ```
    /// # use char_escape::{escaper, UnescapeError};
    /// #
    /// let escaper = escaper! {
    ///     '\n' => 'n',
    ///     '\t' => 't',
    /// };
    ///
    /// let error1 = escaper.unescape(r"\nval\d escape sequence");
    ///
    /// assert_eq!(error1, Err(UnescapeError::Invalid(r"\d".to_string())));
    /// ```
    ///
    /// ... or if the string to escape ends with the escape character.
    ///
    /// ```
    /// # use char_escape::{escaper, UnescapeError};
    /// #
    /// # let escaper = escaper! {
    /// #     '\n' => 'n',
    /// #     '\t' => 't',
    /// # };
    /// #
    /// let error2 = escaper.unescape(r"another failure\");
    ///
    /// assert_eq!(error2, Err(UnescapeError::Incomplete));
    /// ```
    pub fn unescape(&self, s: &str) -> Result<String, UnescapeError> {
        let mut ret = String::with_capacity(s.len());
        let mut previous_was_escape_char = false;
        for c in s.chars() {
            if previous_was_escape_char {
                ret.push(self.unescape_char(c)?);
                previous_was_escape_char = false;
            } else if c == self.escape_char {
                previous_was_escape_char = true;
            } else {
                ret.push(c);
            }
        }

        if previous_was_escape_char {
            Err(UnescapeError::Incomplete)
        } else {
            Ok(ret)
        }
    }

    fn unescape_char(&self, c: char) -> Result<char, UnescapeError> {
        self.rules
            .iter()
            .find(|rule| rule.escaped == c)
            .map_or_else(
                || {
                    Err(UnescapeError::Invalid(
                        [self.escape_char, c].into_iter().collect(),
                    ))
                },
                |rule| Ok(rule.unescaped),
            )
    }

    /// Check if the given string is escaped.
    ///
    // TODO proptest
    /// A string is considered escaped if it contains only valid escape sequences, it contains no
    /// [`char`] that need to be escaped and it doesn't end with the escape character.
    ///
    // TODO proptest
    /// If a string is escaped it is guaranteed that [unescaping](Escaper::unescape) it will never
    /// generate an error.
    ///
    // TODO proptest
    /// A string returned by [`escape()`](Self::escape) will always return true when tested if it
    /// [`is_escaped()`](Self::is_escaped).
    /// 
    /// # Examples
    ///
    /// ```
    /// # use char_escape::escaper;
    /// #
    /// let escaper = escaper! {
    ///     escape_char = '\\',
    ///     rules = [
    ///         '&' => 'a',
    ///         '\\' => 'b',
    ///         '%' => 'm',
    ///         '|' => 'p',
    ///         '/' => 's',
    ///     ],
    /// };
    ///
    /// // contains invalid escape sequence
    /// assert_eq!(escaper.is_escaped(r"\a  \b  \c  \d  \e"), false);
    ///
    /// // ends with the escape character
    /// assert_eq!(escaper.is_escaped(r"\a  \b  \m  \p  \s  \"), false);
    ///
    /// // contains a char that should be escaped
    /// assert_eq!(escaper.is_escaped(r"\a  \b  \m  \|  \s"), false);
    ///
    /// // and finally... the following is escaped
    /// assert_eq!(escaper.is_escaped(r"\a  \b  \m  \p  \s"), true);
    /// ```
    pub fn is_escaped(&self, s: &str) -> bool {
        let mut previous_was_escape_char = false;
        for c in s.chars() {
            #[allow(clippy::collapsible_else_if)]
            if previous_was_escape_char {
                if self
                    .rules
                    .iter()
                    .map(|rule| rule.escaped)
                    .any(|rule_c| rule_c == c)
                {
                    previous_was_escape_char = false;
                } else {
                    // invalid escape sequence => the string is not escaped
                    return false;
                }
            } else {
                if c == self.escape_char {
                    previous_was_escape_char = true;
                } else if self
                    .rules
                    .iter()
                    .map(|rule| rule.unescaped)
                    .any(|needs_to_be_escaped| c == needs_to_be_escaped)
                {
                    // character that needs to be escaped but isn't => the string is not escaped
                    return false;
                }
            }
        }

        !previous_was_escape_char
    }
}

fn contains_escape_char_rule(escape_char: char, rules: &[Rule]) -> bool {
    rules.iter().any(|rule| rule.unescaped == escape_char)
}

/// The error that occurs if unescaping a string fails.
///
/// See also [`Escaper::unescape()`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnescapeError {
    /// Indicates that an invalid escape sequence was encountered.
    ///
    /// The associated [`String`] value is the invalid escape sequence.
    Invalid(String),
    /// Indicates that the string that was to be escaped ended with the escape character.
    Incomplete,
}

impl Display for UnescapeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Invalid(sequence) => write!(f, "invalid escape sequence: {sequence}"),
            Self::Incomplete => write!(f, "incomplete escape sequence"),
        }
    }
}

impl Error for UnescapeError {}

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingEscapeCharRule {}

impl MissingEscapeCharRule {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Display for MissingEscapeCharRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "no escape sequence defined for the escape character")
    }
}

impl Error for MissingEscapeCharRule {}

// TODO test how it handles non-ascii chars
