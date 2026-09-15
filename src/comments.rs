//! Extract comment spans from parsed SQL files.
//!
//! Definitions used throughout this crate:
//! - **leading**: a comment that appears on lines immediately preceding a statement/column
//! - **inline**: a comment that appears after code on the same line (ignored)
//! - **interstitial**: a comment inside a statement (ignored)

use alloc::{borrow::ToOwned, string::String, vec::Vec};

use crate::ast::ParsedSqlSource;

/// Represents a line/column location within a source file.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub struct Location {
    line: u64,
    column: u64,
}

impl Location {
    /// Method for instantiating a new [`Location`]
    ///
    /// # Parameters
    /// - line: the [`u64`] value of the line location
    /// - column: the [`u64`] value of the column location
    #[must_use]
    pub const fn new(line: u64, column: u64) -> Self {
        Self { line, column }
    }

    /// Getter method for getting the line value
    #[must_use]
    pub const fn line(&self) -> u64 {
        self.line
    }

    /// Getter method for getting the column value
    #[must_use]
    pub const fn column(&self) -> u64 {
        self.column
    }
}

impl Default for Location {
    fn default() -> Self {
        Self::new(1, 1)
    }
}

/// Location of a comment, from its opening marker to one column past its last character.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    start: Location,
    end: Location,
}

impl Span {
    /// Method for creating a new instance of the [`Span`] for a
    /// comment's span
    ///
    /// # Parameters
    /// - the [`Location`] where the comment starts in the file
    /// - the [`Location`] where the comment ends in the file
    #[must_use]
    pub const fn new(start: Location, end: Location) -> Self {
        Self { start, end }
    }

    /// Getter for the start location of a [`Span`]
    #[must_use]
    pub const fn start(&self) -> &Location {
        &self.start
    }

    /// Getter for the end location of a [`Span`]
    #[must_use]
    pub const fn end(&self) -> &Location {
        &self.end
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new(Location::default(), Location::default())
    }
}

/// Enum for differentiating comments by single line `--` and
/// multiline `/* */`
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommentKind {
    /// Enum variant for Multiline Comments
    MultiLine,
    /// Enum variant for Single Line Comments
    SingleLine,
}

/// Structure for containing the [`CommentKind`] and the [`Span`] for a comment
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Comment {
    text: String,
    kind: CommentKind,
    span: Span,
}

impl Comment {
    /// Method for making a new comment
    ///
    /// # Parameters
    /// - `kind` where the type of comment is passed as a [`CommentKind`]
    /// - `span` where the [`Span`] of the comment is passed
    #[must_use]
    pub const fn new(text: String, kind: CommentKind, span: Span) -> Self {
        Self { text, kind, span }
    }

    /// Getter method to get the [`CommentKind`]
    #[must_use]
    pub const fn kind(&self) -> &CommentKind {
        &self.kind
    }

    /// Getter method to get the [`Span`] of the comment
    #[must_use]
    pub const fn span(&self) -> &Span {
        &self.span
    }

    /// Getter method that will return the comment content as a [`str`],
    /// regardless of [`CommentKind`]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Consumes the comment and returns its content.
    #[must_use]
    pub fn into_text(self) -> String {
        self.text
    }
}

/// Enum for returning errors withe Comment parsing
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommentError {
    /// Found a multiline comment terminator `*/` without a matching opener `/*`
    UnmatchedMultilineCommentStart {
        /// Returns the location of the terminator found
        location: Location,
    },
    /// Found a multiline comment that is not properly terminated before EOF
    UnterminatedMultiLineComment {
        /// Returns the location of where the multiline comment started
        start: Location,
    },
}

impl core::fmt::Display for CommentError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnmatchedMultilineCommentStart { location } => {
                write!(
                    f,
                    "unmatched block comment start at line {}, column {}",
                    location.line(),
                    location.column()
                )
            }
            Self::UnterminatedMultiLineComment { start } => {
                write!(
                    f,
                    "unterminated block comment with start at line {}, column {}",
                    start.line(),
                    start.column(),
                )
            }
        }
    }
}

impl core::error::Error for CommentError {}

/// Alias for comment results that may return a [`CommentError`]
pub type CommentResult<T> = Result<T, CommentError>;

/// Structure for holding all comments found in the document
#[derive(Debug, Eq, PartialEq)]
pub struct Comments {
    comments: Vec<Comment>,
}

impl Comments {
    /// Method for generating a new [`Comments`] struct, which sorts comments
    /// based on their starting span location
    ///
    /// # Parameters
    /// - `comments`: mutable [`Vec<Comment>`] that will be sorted by span start
    #[must_use]
    pub fn new(mut comments: Vec<Comment>) -> Self {
        // Always keep comments ordered by their span
        comments.sort_by(|a, b| {
            let a_start = a.span().start();
            let b_start = b.span().start();

            a_start
                .line()
                .cmp(&b_start.line())
                .then_with(|| a_start.column().cmp(&b_start.column()))
        });

        Self { comments }
    }

    /// Build all leading comments from a parsed SQL file
    ///
    /// # Parameters
    /// - `file`: the [`ParsedSqlSource`] that needs to be parsed for comments
    ///
    /// # Errors
    /// - Will return [`CommentError::UnmatchedMultilineCommentStart`] if a
    ///   comment does not have an opening `/*`
    /// - Will return [`CommentError::UnterminatedMultiLineComment`] if a
    ///   multiline comment doesn't end before `EOF`
    pub fn parse_all_comments_from_file(file: &ParsedSqlSource) -> CommentResult<Self> {
        Self::scan_comments_with(file.content(), file.string_escapes())
    }

    /// Scans the raw file and collects every comment that is not inline, with
    /// quotes escaped the standard way, by doubling them.
    ///
    /// Comment markers inside string literals, quoted identifiers and dollar
    /// quoted strings are text, block comments nest, and a comment preceded by
    /// code on its own line is inline and therefore dropped.
    ///
    /// # Parameters
    /// - `src` which is the `SQL` file content as a [`str`]
    ///
    /// # Errors
    /// - `UnmatchedMultilineCommentStart` : will return error if a `*/` appears
    ///   outside of a block comment
    /// - `UnterminatedMultiLineComment` : will return error if a block comment
    ///   is still open at `EOF`
    pub fn scan_comments(src: &str) -> CommentResult<Self> {
        Self::scan_comments_with(src, StringEscapes::Doubled)
    }

    /// Scans the raw file the way `escapes` says its string literals are written.
    ///
    /// # Parameters
    /// - `src` which is the `SQL` file content as a [`str`]
    /// - `escapes` the [`StringEscapes`] of the dialect the source is written in
    ///
    /// # Errors
    /// - `UnmatchedMultilineCommentStart` : will return error if a `*/` appears
    ///   outside of a block comment
    /// - `UnterminatedMultiLineComment` : will return error if a block comment
    ///   is still open at `EOF`
    pub fn scan_comments_with(src: &str, escapes: StringEscapes) -> CommentResult<Self> {
        let mut scanner = Scanner::new(src, escapes);
        let mut chars = src.char_indices().peekable();
        while let Some((offset, character)) = chars.next() {
            let peeked = chars.peek().map(|&(_, next)| next);
            if character == '\r' && peeked == Some('\n') {
                continue;
            }
            if character == '\n' {
                scanner.newline();
            } else {
                scanner.step(offset, character, peeked, &mut chars)?;
            }
        }
        scanner.finish()
    }

    /// Getter method for retrieving the Vec of [`Comment`]
    #[must_use]
    pub fn comments(&self) -> &[Comment] {
        &self.comments
    }

    /// Finds a single comment before a specific line or returns none
    ///
    /// # Parameters
    /// - [`Comments`] object
    /// - An `u64` value representing the desired line to check above.
    #[must_use]
    pub fn leading_comment(&self, line: u64) -> Option<&Comment> {
        let before = self.comments.partition_point(|c| c.span().start().line() < line);
        self.comments[..before].last().filter(|c| c.span().end().line() + 1 == line)
    }

    /// Finds leading comments before specific line based on [`LeadingCommentCapture`] preference
    ///
    /// Several comments on one line all belong to that line, so all of them are
    /// returned when the line is part of the captured run.
    ///
    /// # Parameters
    /// - [`Comments`] object
    /// - An `u64` value representing the desired line to check above.
    /// - [`LeadingCommentCapture`] preference
    #[must_use]
    pub fn leading_comments(&self, line: u64, capture: LeadingCommentCapture) -> Self {
        Self { comments: self.leading_run(line, capture).to_vec() }
    }

    /// Collapses the leading comments of `line` into a single [`Comment`].
    #[must_use]
    pub fn leading_doc(
        &self,
        line: u64,
        capture: LeadingCommentCapture,
        flatten: MultiFlatten<'_>,
    ) -> Option<Comment> {
        collapse(self.leading_run(line, capture), flatten)
    }

    /// Returns the run of comments that lead `line`, nearest one last.
    ///
    /// The run covers the block of lines directly above `line`, so comments
    /// sharing a line are kept together, and it stops where `capture` says.
    fn leading_run(&self, line: u64, capture: LeadingCommentCapture) -> &[Comment] {
        let end = self.comments.partition_point(|c| c.span().start().line() < line);
        let Some(mut index) = end.checked_sub(1) else { return &[] };
        if self.comments[index].span().end().line() + 1 != line {
            return &[];
        }
        if capture == LeadingCommentCapture::SingleNearest {
            return &self.comments[index..end];
        }
        let mut seen_multiline = *self.comments[index].kind() == CommentKind::MultiLine;
        while index > 0 {
            let candidate = &self.comments[index - 1];
            let accepted_line = self.comments[index].span().start().line();
            let candidate_end = candidate.span().end().line();
            let same_line = candidate_end == accepted_line;
            if !same_line && candidate_end + 1 != accepted_line {
                break;
            }
            if !same_line
                && capture == LeadingCommentCapture::AllSingleOneMulti
                && *candidate.kind() == CommentKind::MultiLine
            {
                if seen_multiline {
                    break;
                }
                seen_multiline = true;
            }
            index -= 1;
        }
        &self.comments[index..end]
    }

    /// Collapse this collection of comments and separate each comment with `\n` as a single [`Comment`].
    #[must_use]
    pub fn collapse_comments(self, flatten: MultiFlatten) -> Option<Comment> {
        collapse(&self.comments, flatten)
    }
}

/// Joins `comments` into one [`Comment`] spanning all of them.
fn collapse(comments: &[Comment], flatten: MultiFlatten) -> Option<Comment> {
    let (first, rest) = comments.split_first()?;
    let Some((_, _)) = rest.split_first() else {
        return Some(Comment::new(
            flatten_lines(first.text(), flatten),
            first.kind().clone(),
            *first.span(),
        ));
    };

    let mut text = first.text().to_owned();
    let mut end = *first.span().end();
    for comment in rest {
        text.push('\n');
        text.push_str(comment.text());
        end = *comment.span().end();
    }

    Some(Comment::new(
        flatten_lines(&text, flatten),
        CommentKind::MultiLine,
        Span::new(*first.span().start(), end),
    ))
}

fn flatten_lines(lines: &str, flatten: MultiFlatten) -> String {
    let mut out = String::new();
    let sep = match flatten {
        MultiFlatten::FlattenWithNone => String::new(),
        MultiFlatten::NoFlat => return lines.to_owned(),
        MultiFlatten::Flatten(chars) => chars.to_owned(),
    };
    for (i, line) in lines.lines().enumerate() {
        if i > 0 {
            out.push_str(&sep);
        }
        out.push_str(line);
    }
    out
}

/// Lexer state while scanning a source for comments.
#[derive(Clone, Copy, Eq, PartialEq)]
enum ScanState<'a> {
    Code,
    Single,
    Block,
    Quoted(char),
    DollarQuoted(&'a str),
}

/// Character iterator used by [`Scanner`].
type Chars<'a> = core::iter::Peekable<core::str::CharIndices<'a>>;

/// Collects the comments of one source, tracking where the scan currently is.
struct Scanner<'a> {
    src: &'a str,
    escapes: StringEscapes,
    quote_escapes: StringEscapes,
    previous_code: Option<char>,
    comments: Vec<Comment>,
    state: ScanState<'a>,
    buf: String,
    line: u64,
    col: u64,
    start: Location,
    depth: u32,
    inline: bool,
    code_before: bool,
}

impl<'a> Scanner<'a> {
    /// Starts a scan at the first character of `src`.
    fn new(src: &'a str, escapes: StringEscapes) -> Self {
        Self {
            src,
            escapes,
            quote_escapes: escapes,
            previous_code: None,
            comments: Vec::new(),
            state: ScanState::Code,
            buf: String::new(),
            line: 1,
            col: 1,
            start: Location::default(),
            depth: 0,
            inline: false,
            code_before: false,
        }
    }

    /// Ends the current line, closing a single line comment with it.
    fn newline(&mut self) {
        match self.state {
            ScanState::Single => self.close_single(),
            ScanState::Block => self.buf.push('\n'),
            ScanState::Code | ScanState::Quoted(_) | ScanState::DollarQuoted(_) => {}
        }
        self.line += 1;
        self.col = 1;
        self.code_before = false;
        self.previous_code = None;
    }

    /// Consumes one character in the current state.
    fn step(
        &mut self,
        offset: usize,
        character: char,
        peeked: Option<char>,
        chars: &mut Chars<'a>,
    ) -> CommentResult<()> {
        match self.state {
            ScanState::Code => return self.code(offset, character, peeked, chars),
            ScanState::Single => {
                self.buf.push(character);
                self.col += 1;
            }
            ScanState::Block => self.block(character, peeked, chars),
            ScanState::Quoted(delimiter) => self.quoted(delimiter, character, peeked, chars),
            ScanState::DollarQuoted(tag) => self.dollar_quoted(tag, offset, character, chars),
        }
        Ok(())
    }

    /// Consumes one character outside of comments and quoted text.
    fn code(
        &mut self,
        offset: usize,
        character: char,
        peeked: Option<char>,
        chars: &mut Chars<'a>,
    ) -> CommentResult<()> {
        match character {
            '-' if peeked == Some('-') => {
                chars.next();
                self.open(ScanState::Single);
            }
            '/' if peeked == Some('*') => {
                chars.next();
                self.depth = 1;
                self.open(ScanState::Block);
            }
            '*' if peeked == Some('/') => {
                return Err(CommentError::UnmatchedMultilineCommentStart {
                    location: Location::new(self.line, self.col),
                });
            }
            '\'' | '"' | '`' => {
                self.quote_escapes =
                    if character == '\'' && matches!(self.previous_code, Some('e' | 'E')) {
                        StringEscapes::Backslash
                    } else {
                        self.escapes
                    };
                self.state = ScanState::Quoted(character);
                self.code_before = true;
                self.col += 1;
            }
            '$' => {
                if let Some(tag) = dollar_tag(self.src.get(offset..).unwrap_or_default()) {
                    self.state = ScanState::DollarQuoted(tag);
                    self.skip_tag(tag, chars);
                }
                self.code_before = true;
                self.col += 1;
            }
            _ => {
                self.code_before |= !character.is_whitespace();
                self.col += 1;
            }
        }
        self.previous_code = Some(character);
        Ok(())
    }

    /// Consumes one character inside a block comment.
    fn block(&mut self, character: char, peeked: Option<char>, chars: &mut Chars<'a>) {
        match character {
            '/' if peeked == Some('*') => {
                chars.next();
                self.depth += 1;
                self.buf.push_str("/*");
                self.col += 2;
            }
            '*' if peeked == Some('/') => {
                chars.next();
                self.depth -= 1;
                if self.depth == 0 {
                    self.close_block();
                } else {
                    self.buf.push_str("*/");
                }
                self.col += 2;
            }
            _ => {
                self.buf.push(character);
                self.col += 1;
            }
        }
    }

    /// Consumes one character inside a string literal or quoted identifier.
    fn quoted(
        &mut self,
        delimiter: char,
        character: char,
        peeked: Option<char>,
        chars: &mut Chars<'a>,
    ) {
        if character == '\\' && delimiter != '`' && self.quote_escapes == StringEscapes::Backslash {
            match chars.next() {
                Some((_, '\n')) => {
                    self.col += 1;
                    self.newline();
                }
                Some(_) => self.col += 2,
                None => self.col += 1,
            }
            return;
        }
        if character == delimiter {
            if peeked == Some(delimiter) {
                chars.next();
                self.col += 1;
            } else {
                self.state = ScanState::Code;
                self.code_before = true;
            }
        }
        self.col += 1;
    }

    /// Consumes one character inside a dollar quoted string.
    fn dollar_quoted(&mut self, tag: &str, offset: usize, character: char, chars: &mut Chars<'a>) {
        if character == '$' && self.src.get(offset..).is_some_and(|rest| rest.starts_with(tag)) {
            self.state = ScanState::Code;
            self.code_before = true;
            self.skip_tag(tag, chars);
        }
        self.col += 1;
    }

    /// Opens a comment at the current location.
    fn open(&mut self, state: ScanState<'a>) {
        self.state = state;
        self.start = Location::new(self.line, self.col);
        self.inline = self.code_before;
        self.buf.clear();
        self.col += 2;
    }

    /// Records the single line comment that ends here.
    fn close_single(&mut self) {
        if !self.inline {
            self.comments.push(Comment::new(
                self.buf.trim().to_owned(),
                CommentKind::SingleLine,
                Span::new(self.start, Location::new(self.line, self.col)),
            ));
        }
        self.buf.clear();
        self.state = ScanState::Code;
    }

    /// Records the block comment whose terminator starts at the current column.
    fn close_block(&mut self) {
        if !self.inline {
            self.comments.push(Comment::new(
                normalize_block(&self.buf),
                CommentKind::MultiLine,
                Span::new(self.start, Location::new(self.line, self.col + 2)),
            ));
        }
        self.buf.clear();
        self.state = ScanState::Code;
    }

    /// Consumes the remaining characters of a dollar quote tag.
    fn skip_tag(&mut self, tag: &str, chars: &mut Chars<'a>) {
        for _ in 1..tag.chars().count() {
            chars.next();
            self.col += 1;
        }
    }

    /// Finishes the scan at `EOF`.
    fn finish(mut self) -> CommentResult<Comments> {
        match self.state {
            ScanState::Single => self.close_single(),
            ScanState::Block => {
                return Err(CommentError::UnterminatedMultiLineComment { start: self.start });
            }
            ScanState::Code | ScanState::Quoted(_) | ScanState::DollarQuoted(_) => {}
        }
        Ok(Comments { comments: self.comments })
    }
}

/// Returns the dollar quote tag, delimiters included, that `rest` opens with.
///
/// A tag follows the rules of an unquoted identifier, so it never starts with a
/// digit, which keeps positional parameters such as `$1` out of the scan.
fn dollar_tag(rest: &str) -> Option<&str> {
    let mut end = 1;
    for (position, character) in rest.get(1..)?.char_indices() {
        if character == '$' {
            return rest.get(..=end);
        }
        let identifier_start = character.is_alphabetic() || character == '_';
        let identifier_part = identifier_start || character.is_numeric();
        if position == 0 && !identifier_start || !identifier_part {
            return None;
        }
        end += character.len_utf8();
    }
    None
}

/// Trims every line of a block comment body.
fn normalize_block(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    for (index, line) in body.lines().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(line.trim());
    }
    out
}

/// How the source being scanned escapes a quote inside a string literal.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StringEscapes {
    /// Standard SQL, where a quote is escaped by doubling it.
    #[default]
    Doubled,
    /// Dialects such as `MySQL`, where a backslash escapes the character after it.
    Backslash,
}

/// Controls how leading comments are captured for a statement.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum LeadingCommentCapture {
    /// Capture only the single nearest leading comment.
    #[default]
    SingleNearest,
    /// Capture all contiguous leading comments, stopping at the first blank line.
    AllLeading,
    /// Capture all contiguous single-line or at most one multi-line leading comments.
    AllSingleOneMulti,
}

/// Enum for multiline comment flattening.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MultiFlatten<'a> {
    /// Default option, retains multiline structure with `\n`
    #[default]
    NoFlat,
    /// Sets multiline comments to be flattened and combined without adding formatting
    FlattenWithNone,
    /// Will flatten comments and amend the content of [`String`] to the end of the former leading lines
    Flatten(&'a str),
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    use alloc::{
        borrow::ToOwned,
        boxed::Box,
        format,
        string::{String, ToString},
        vec,
        vec::Vec,
    };

    use crate::comments::{
        Comment, CommentError, CommentKind, Comments, LeadingCommentCapture, Location, Span,
        StringEscapes,
    };

    fn scanned_texts(src: &str) -> Vec<String> {
        Comments::scan_comments(src)
            .unwrap_or_else(|error| panic!("scan failed: {error}"))
            .comments()
            .iter()
            .map(|comment| comment.text().to_owned())
            .collect()
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn comment_markers_inside_string_literals_are_text() {
        assert!(scanned_texts("SELECT 'a -- b', \"c /* d\", `e -- f`;").is_empty());
        assert_eq!(scanned_texts("SELECT 'it''s -- fine';\n-- real\n"), vec!["real".to_owned()]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn backslash_escaped_quote_does_not_swallow_later_comments() {
        let src = "SELECT 'a\\'b';\n-- real\n";
        assert_eq!(
            Comments::scan_comments_with(src, StringEscapes::Backslash)
                .unwrap_or_else(|error| panic!("scan failed: {error}"))
                .comments()
                .iter()
                .map(|comment| comment.text().to_owned())
                .collect::<Vec<_>>(),
            vec!["real".to_owned()]
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn comment_markers_inside_dollar_quotes_are_text() {
        let src = "CREATE FUNCTION f() AS $body$ -- inner\nSELECT 1 $body$;\n-- real\n";
        assert_eq!(scanned_texts(src), vec!["real".to_owned()]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn comment_after_a_closing_dollar_quote_is_inline() {
        let src = "CREATE FUNCTION f() AS $body$\nSELECT 1;\n$body$ -- after\n-- real\n";
        assert_eq!(scanned_texts(src), vec!["real".to_owned()]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn an_escape_string_literal_escapes_with_a_backslash() {
        assert_eq!(
            scanned_texts("SELECT E'a\\'b';\n-- real\n"),
            vec!["real".to_owned()],
            "an E prefixed literal takes backslash escapes in every dialect"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn a_positional_parameter_does_not_open_a_dollar_quote() {
        assert_eq!(
            scanned_texts("SELECT $1$ FROM t;\n-- real\n"),
            vec!["real".to_owned()],
            "a tag may not start with a digit"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn block_comments_nest() {
        assert_eq!(
            scanned_texts("/* outer /* inner */ still outer */\n"),
            vec!["outer /* inner */ still outer".to_owned()]
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn inline_comments_are_dropped() {
        assert!(scanned_texts("SELECT 1; -- trailing\n").is_empty());
        assert!(scanned_texts("CREATE TABLE t ( /* trailing */\n").is_empty());
        assert_eq!(scanned_texts("  -- indented\n"), vec!["indented".to_owned()]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn comment_spans_end_one_column_past_the_comment() {
        let single = Comments::scan_comments("-- ab\n")
            .unwrap_or_else(|error| panic!("scan failed: {error}"));
        assert_eq!(
            single.comments()[0].span(),
            &Span::new(Location::new(1, 1), Location::new(1, 6))
        );

        let block = Comments::scan_comments("/* ab */\n")
            .unwrap_or_else(|error| panic!("scan failed: {error}"));
        assert_eq!(
            block.comments()[0].span(),
            &Span::new(Location::new(1, 1), Location::new(1, 9))
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn unterminated_block_comment_reports_its_start() {
        assert_eq!(
            Comments::scan_comments("SELECT 1;\n  /* open"),
            Err(CommentError::UnterminatedMultiLineComment { start: Location::new(2, 3) })
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn every_comment_on_a_captured_line_is_captured() {
        let comments = Comments::scan_comments("/* one */ /* two */\n")
            .unwrap_or_else(|error| panic!("scan failed: {error}"));
        let captured = comments.leading_comments(2, LeadingCommentCapture::AllLeading);
        assert_eq!(
            captured.comments().iter().map(|c| c.text().to_owned()).collect::<Vec<_>>(),
            vec!["one".to_owned(), "two".to_owned()]
        );
        assert_eq!(
            comments
                .leading_comments(2, LeadingCommentCapture::SingleNearest)
                .comments()
                .iter()
                .map(|c| c.text().to_owned())
                .collect::<Vec<_>>(),
            vec!["two".to_owned()]
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn parse_comments() -> Result<(), Box<dyn std::error::Error>> {
        use sqlparser::dialect::GenericDialect;

        use crate::{ast::ParsedSqlSourceSet, comments::Comments, source::SqlSource};
        let base = std::env::temp_dir().join("all_sql_files");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base)?;
        let file1 = base.join("with_single_line_comments.sql");
        std::fs::File::create(&file1)?;
        std::fs::write(&file1, single_line_comments_sql())?;
        let file2 = base.join("with_multiline_comments.sql");
        std::fs::File::create(&file2)?;
        std::fs::write(&file2, multiline_comments_sql())?;
        let file3 = base.join("with_mixed_comments.sql");
        std::fs::File::create(&file3)?;
        std::fs::write(&file3, mixed_comments_sql())?;
        let file4 = base.join("without_comments.sql");
        std::fs::File::create(&file4)?;
        std::fs::write(&file4, no_comments_sql())?;
        let set = SqlSource::sql_sources(&base, &[])?;
        let parsed_set = ParsedSqlSourceSet::parse_all::<GenericDialect>(set)?;

        for file in parsed_set.sources() {
            let parsed_comments = Comments::parse_all_comments_from_file(file)?;
            let filename = file
                .source()
                .path()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .ok_or("Should have a file name")?;

            match filename {
                "with_single_line_comments.sql" => {
                    assert_parsed_comments_eq(&parsed_comments, expected_single_line_comments());
                }
                "with_multiline_comments.sql" => {
                    assert_parsed_comments_eq(&parsed_comments, expected_multiline_comments());
                }
                "with_mixed_comments.sql" => {
                    assert_parsed_comments_eq(&parsed_comments, expected_mixed_comments());
                }
                "without_comments.sql" => {
                    assert!(parsed_comments.comments().is_empty());
                }
                other => {
                    unreachable!(
                        "unexpected test file {other}; directory should only contain known test files"
                    );
                }
            }
        }
        let _ = std::fs::remove_dir_all(&base);
        Ok(())
    }
    #[cfg(feature = "std")]
    fn assert_parsed_comments_eq(parsed: &Comments, expected: &[&str]) {
        let comments = parsed.comments();
        assert_eq!(
            expected.len(),
            comments.len(),
            "mismatched comment count (expected {}, got {})",
            expected.len(),
            comments.len()
        );

        for (i, comment) in comments.iter().enumerate() {
            assert_eq!(expected[i], comment.text(), "comment at index {i} did not match");
        }
    }
    #[cfg(feature = "std")]
    fn single_line_comments_sql() -> &'static str {
        "-- Users table stores user account information
CREATE TABLE users (
    -- Primary key
    id INTEGER PRIMARY KEY,
    -- Username for login
    username VARCHAR(255) NOT NULL,
    -- Email address
    email VARCHAR(255) UNIQUE NOT NULL,
    -- When the user registered
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Posts table stores blog posts
CREATE TABLE posts (
    -- Primary key
    id INTEGER PRIMARY KEY,
    -- Post title
    title VARCHAR(255) NOT NULL,
    -- Foreign key linking to users
    user_id INTEGER NOT NULL,
    -- Main body text
    body TEXT NOT NULL,
    -- When the post was created
    published_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);"
    }
    #[cfg(feature = "std")]
    fn multiline_comments_sql() -> &'static str {
        r"/* Users table stores user account information 
multiline */
CREATE TABLE users (
    /* Primary key 
    multiline */
    id INTEGER PRIMARY KEY,
    /* Username for login 
    multiline */
    username VARCHAR(255) NOT NULL,
    /* Email address 
    multiline */
    email VARCHAR(255) UNIQUE NOT NULL,
    /* When the user registered 
    multiline */
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

/* Posts table stores blog posts 
multiline */
CREATE TABLE posts (
    /* Primary key 
    multiline */
    id INTEGER PRIMARY KEY,
    /* Post title 
    multiline */
    title VARCHAR(255) NOT NULL,
    /* Foreign key linking to users 
    multiline */
    user_id INTEGER NOT NULL,
    /* Main body text 
    multiline */
    body TEXT NOT NULL,
    /* When the post was created 
    multiline */
    published_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);"
    }
    #[cfg(feature = "std")]
    fn no_comments_sql() -> &'static str {
        "CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE posts (
    id INTEGER PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    user_id INTEGER NOT NULL,
    body TEXT NOT NULL,
    published_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);"
    }
    #[cfg(feature = "std")]
    fn mixed_comments_sql() -> &'static str {
        "-- interstitial Comment above statements (should be ignored)

/* Users table stores user account information */
CREATE TABLE users ( /* users interstitial comment 
(should be ignored) */
    -- Primary key
    id INTEGER PRIMARY KEY, -- Id comment that is interstitial (should be ignored)
    /* Username for login */
    username VARCHAR(255) NOT NULL,
    -- Email address
    email VARCHAR(255) UNIQUE NOT NULL,
    /* When the user registered */
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

/* Posts table stores blog posts */
CREATE TABLE posts (
    -- Primary key
    id INTEGER PRIMARY KEY,
    /* Post title */
    title VARCHAR(255) NOT NULL,
    -- Foreign key linking to users
    user_id INTEGER NOT NULL,
    /* Main body text */
    body TEXT NOT NULL,
    -- When the post was created
    published_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
"
    }
    #[cfg(feature = "std")]
    fn expected_single_line_comments() -> &'static [&'static str] {
        &[
            "Users table stores user account information",
            "Primary key",
            "Username for login",
            "Email address",
            "When the user registered",
            "Posts table stores blog posts",
            "Primary key",
            "Post title",
            "Foreign key linking to users",
            "Main body text",
            "When the post was created",
        ]
    }
    #[cfg(feature = "std")]
    fn expected_multiline_comments() -> &'static [&'static str] {
        &[
            "Users table stores user account information\nmultiline",
            "Primary key\nmultiline",
            "Username for login\nmultiline",
            "Email address\nmultiline",
            "When the user registered\nmultiline",
            "Posts table stores blog posts\nmultiline",
            "Primary key\nmultiline",
            "Post title\nmultiline",
            "Foreign key linking to users\nmultiline",
            "Main body text\nmultiline",
            "When the post was created\nmultiline",
        ]
    }
    #[cfg(feature = "std")]
    fn expected_mixed_comments() -> &'static [&'static str] {
        &[
            "interstitial Comment above statements (should be ignored)",
            "Users table stores user account information",
            "Primary key",
            "Username for login",
            "Email address",
            "When the user registered",
            "Posts table stores blog posts",
            "Primary key",
            "Post title",
            "Foreign key linking to users",
            "Main body text",
            "When the post was created",
        ]
    }

    #[cfg(feature = "std")]
    #[test]
    fn single_line_comment_spans_are_correct() -> Result<(), Box<dyn std::error::Error>> {
        use sqlparser::dialect::GenericDialect;

        use crate::{ast::ParsedSqlSourceSet, source::SqlSource};
        let base = std::env::temp_dir().join("single_line_spans");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base)?;
        let file = base.join("single.sql");
        std::fs::File::create(&file)?;
        std::fs::write(&file, single_line_comments_sql())?;
        let set = SqlSource::sql_sources(&base, &[])?;
        let parsed_set = ParsedSqlSourceSet::parse_all::<GenericDialect>(set)?;
        let file = parsed_set
            .sources()
            .iter()
            .find(|f| {
                f.source()
                    .path()
                    .and_then(|p| p.to_str())
                    .is_some_and(|p| p.ends_with("single.sql"))
            })
            .ok_or("single.sql should be present")?;

        let comments = Comments::parse_all_comments_from_file(file)?;
        let comments = comments.comments();
        assert_eq!(comments.len(), 11);
        let first = &comments[0];
        assert_eq!(first.text(), "Users table stores user account information");
        assert_eq!(first.span().start(), &Location::new(1, 1));
        assert_eq!(first.span().end(), &Location::new(1, 47));
        let primary_key = &comments[1];
        assert_eq!(primary_key.text(), "Primary key");
        assert_eq!(primary_key.span().start(), &Location::new(3, 5));
        assert_eq!(primary_key.span().end(), &Location::new(3, 19));
        assert!(
            primary_key.span().end().column() > primary_key.span().start().column(),
            "end column should be after start column",
        );
        let _ = std::fs::remove_dir_all(&base);
        Ok(())
    }

    #[cfg(feature = "std")]
    #[test]
    fn multiline_comment_spans_are_correct() -> Result<(), Box<dyn std::error::Error>> {
        use sqlparser::dialect::GenericDialect;

        use crate::{ast::ParsedSqlSourceSet, source::SqlSource};
        let base = std::env::temp_dir().join("multi_line_spans");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base)?;
        let file = base.join("multi.sql");
        std::fs::File::create(&file)?;
        std::fs::write(&file, multiline_comments_sql())?;
        let set = SqlSource::sql_sources(&base, &[])?;
        let parsed_set = ParsedSqlSourceSet::parse_all::<GenericDialect>(set)?;
        let file = parsed_set
            .sources()
            .iter()
            .find(|f| {
                f.source().path().and_then(|p| p.to_str()).is_some_and(|p| p.ends_with("multi.sql"))
            })
            .ok_or("multi.sql should be present")?;

        let comments = Comments::parse_all_comments_from_file(file)?;
        let comments = comments.comments();
        assert_eq!(comments.len(), 11);
        let first = &comments[0];
        assert_eq!(first.text(), "Users table stores user account information\nmultiline");
        assert_eq!(first.span().start(), &Location::new(1, 1));
        assert_eq!(first.span().end().line(), 2);
        assert!(
            first.span().end().column() > first.span().start().column(),
            "end column should be after start column for first multiline comment",
        );
        let primary_key = &comments[1];
        assert_eq!(primary_key.text(), "Primary key\nmultiline");
        assert_eq!(primary_key.span().start(), &Location::new(4, 5));
        assert_eq!(primary_key.span().end().line(), 5);
        assert!(
            primary_key.span().end().column() > primary_key.span().start().column(),
            "end column should be after start column for primary key multiline comment",
        );
        let _ = std::fs::remove_dir_all(&base);
        Ok(())
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_comment_error() {
        let unterminated =
            CommentError::UnterminatedMultiLineComment { start: Location::default() };
        let location = Location { line: 1, column: 1 };
        let expected = format!(
            "unterminated block comment with start at line {}, column {}",
            location.line(),
            location.column()
        );
        assert_eq!(unterminated.to_string(), expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_comments() {
        let comment_vec = vec![
            Comment::new(
                "a comment".to_owned(),
                CommentKind::SingleLine,
                Span { start: Location::new(1, 1), end: Location::new(1, 12) },
            ),
            Comment::new(
                "a second comment".to_owned(),
                CommentKind::SingleLine,
                Span { start: Location::new(1, 1), end: Location::new(2, 19) },
            ),
        ];
        let length = comment_vec.len();
        let comments = Comments::new(comment_vec.clone());
        assert_eq!(comments.comments().len(), length);
        for (i, comment) in comments.comments().iter().enumerate() {
            assert_eq!(comment.text(), comment_vec[i].text());
            assert_eq!(comment.span().start(), comment_vec[i].span().start());
            assert_eq!(comment.span().end(), comment_vec[i].span().end());
        }
    }

    fn texts(v: &Comments) -> Vec<String> {
        v.comments().iter().map(|c| c.text().to_owned()).collect()
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn leading_comment_capture_default_is_single_nearest() {
        assert_eq!(LeadingCommentCapture::default(), LeadingCommentCapture::SingleNearest);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn leading_comments_single_nearest_and_all_leading_basic_runover()
    -> Result<(), Box<dyn core::error::Error>> {
        let src = "\
-- c1
-- c2
CREATE TABLE t (id INTEGER);
";
        let parsed = Comments::scan_comments(src)?;
        let single = parsed.leading_comments(3, LeadingCommentCapture::SingleNearest);
        assert_eq!(texts(&single), vec!["c2".to_owned()]);

        let all = parsed.leading_comments(3, LeadingCommentCapture::AllLeading);
        assert_eq!(texts(&all), vec!["c1".to_owned(), "c2".to_owned()]);

        Ok(())
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn leading_comments_all_leading_stops_at_blank_line() -> Result<(), Box<dyn core::error::Error>>
    {
        let src = "\
-- c1

-- c2
CREATE TABLE t (id INTEGER);
";
        let parsed = Comments::scan_comments(src)?;
        let all = parsed.leading_comments(4, LeadingCommentCapture::AllLeading);
        assert_eq!(texts(&all), vec!["c2".to_owned()]);

        Ok(())
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn leading_comments_all_single_one_multi_collects_singles_and_one_multiline()
    -> Result<(), Box<dyn core::error::Error>> {
        let src = "\
/* m
m */
-- s1
-- s2
CREATE TABLE t (id INTEGER);
";
        let parsed = Comments::scan_comments(src)?;
        let got = parsed.leading_comments(5, LeadingCommentCapture::AllSingleOneMulti);
        assert_eq!(texts(&got), vec!["m\nm".to_owned(), "s1".to_owned(), "s2".to_owned(),]);

        Ok(())
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn leading_comments_all_single_one_multi_stops_before_second_multiline()
    -> Result<(), Box<dyn core::error::Error>> {
        let src = "\
/* m1 */
/* m2 */
-- s1
CREATE TABLE t (id INTEGER);
";
        let parsed = Comments::scan_comments(src)?;
        let got = parsed.leading_comments(4, LeadingCommentCapture::AllSingleOneMulti);
        assert_eq!(texts(&got), vec!["m2".to_owned(), "s1".to_owned()]);

        Ok(())
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn leading_comments_single_nearest_can_return_multiline()
    -> Result<(), Box<dyn core::error::Error>> {
        let src = "\
/* hello
world */
CREATE TABLE t (id INTEGER);
";
        let parsed = Comments::scan_comments(src)?;
        let got = parsed.leading_comments(3, LeadingCommentCapture::SingleNearest);
        assert_eq!(texts(&got), vec!["hello\nworld".to_owned()]);

        Ok(())
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn collapse_comments_empty_returns_none() {
        let comments = Comments::new(vec![]);
        assert!(comments.collapse_comments(crate::comments::MultiFlatten::NoFlat).is_none());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn collapse_comments_single_returns_same_comment() {
        let c = Comment::new(
            "solo".to_owned(),
            CommentKind::SingleLine,
            Span::new(Location::new(10, 3), Location::new(10, 11)),
        );
        let comments = Comments::new(vec![c]);

        let collapsed = comments
            .collapse_comments(crate::comments::MultiFlatten::NoFlat)
            .unwrap_or_else(|| panic!("should return a comment"));
        assert_eq!(collapsed.text(), "solo");
        assert_eq!(collapsed.kind(), &CommentKind::SingleLine);
        assert_eq!(collapsed.span(), &Span::new(Location::new(10, 3), Location::new(10, 11)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn collapse_comments_multiple_joins_text_and_expands_span_and_sets_multiline_kind() {
        let c1 = Comment::new(
            "a".to_owned(),
            CommentKind::SingleLine,
            Span::new(Location::new(1, 1), Location::new(1, 6)),
        );
        let c2 = Comment::new(
            "b".to_owned(),
            CommentKind::SingleLine,
            Span::new(Location::new(2, 1), Location::new(2, 6)),
        );
        let c3 = Comment::new(
            "c".to_owned(),
            CommentKind::MultiLine,
            Span::new(Location::new(3, 1), Location::new(4, 3)),
        );

        let comments = Comments::new(vec![c1, c2, c3]);

        let collapsed = comments
            .collapse_comments(crate::comments::MultiFlatten::NoFlat)
            .unwrap_or_else(|| panic!("should collapse"));
        assert_eq!(collapsed.text(), "a\nb\nc");
        assert_eq!(collapsed.kind(), &CommentKind::MultiLine);
        assert_eq!(collapsed.span(), &Span::new(Location::new(1, 1), Location::new(4, 3)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn collapse_comments_with_leading_comments_allleading_collapses_correctly()
    -> Result<(), Box<dyn core::error::Error>> {
        let src = "\
-- c1
-- c2
CREATE TABLE t (id INTEGER);
";
        let parsed = Comments::scan_comments(src)?;

        let leading = parsed.leading_comments(3, LeadingCommentCapture::AllLeading);
        assert_eq!(texts(&leading), vec!["c1".to_owned(), "c2".to_owned()]);

        let collapsed = leading
            .collapse_comments(crate::comments::MultiFlatten::NoFlat)
            .unwrap_or_else(|| panic!("should collapse"));
        assert_eq!(collapsed.text(), "c1\nc2");
        assert_eq!(collapsed.kind(), &CommentKind::MultiLine);

        // Span sanity: starts at first comment start, ends at second comment end.
        assert_eq!(*collapsed.span().start(), Location::new(1, 1));
        assert_eq!(collapsed.span().end().line(), 2);

        Ok(())
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn collapse_comments_with_leading_comments_single_nearest_preserves_kind()
    -> Result<(), Box<dyn core::error::Error>> {
        let src = "\
-- c1
-- c2
CREATE TABLE t (id INTEGER);
";
        let parsed = Comments::scan_comments(src)?;
        let leading = parsed.leading_comments(3, LeadingCommentCapture::SingleNearest);
        assert_eq!(texts(&leading), vec!["c2".to_owned()]);

        let collapsed = leading
            .collapse_comments(crate::comments::MultiFlatten::NoFlat)
            .unwrap_or_else(|| panic!("should collapse"));
        assert_eq!(collapsed.text(), "c2");
        assert_eq!(collapsed.kind(), &CommentKind::SingleLine);

        Ok(())
    }
    use crate::comments::flatten_lines;
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_flatten_lines_behavior() {
        let input = "a\nb\nc";
        let no_sep = flatten_lines(input, crate::comments::MultiFlatten::FlattenWithNone);
        assert_eq!(no_sep, "abc");
        let dash_sep = flatten_lines(input, crate::comments::MultiFlatten::Flatten(" - "));
        assert_eq!(dash_sep, "a - b - c");
        let single = flatten_lines("solo", crate::comments::MultiFlatten::Flatten("XXX"));
        assert_eq!(single, "solo");
    }
}
