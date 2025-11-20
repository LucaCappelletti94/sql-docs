//! Module for crawling the SQL documents based on the parser and
//! parsing/extracting the leading comments.
//!
//! *leading comment* a comment that
//! precedes an SQL Statement.

/// Structure for holding a location in the file. Assumes file is first split by
/// lines and then split by characters (column)
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

/// A structure for holding the span of comments found
pub struct CommentSpan {
    start: Location,
    end: Location,
}

impl CommentSpan {
    /// Method for creating a new instance of the [`CommentSpan`] for a
    /// comment's span
    ///
    /// # Parameters
    /// - the [`Location`] where the comment starts in the file
    /// - the [`Location`] where the comment ends in the file
    #[must_use]
    pub const fn new(start: Location, end: Location) -> Self {
        Self { start, end }
    }

    /// Getter for the start location of a [`CommentSpan`]
    #[must_use]
    pub const fn start(&self) -> &Location {
        &self.start
    }

    /// Getter for the end location of a [`CommentSpan`]
    #[must_use]
    pub const fn end(&self) -> &Location {
        &self.end
    }
}

/// Structure that holds the comment along with its location in the file
pub struct Comment {
    comment: String,
    span: CommentSpan,
}

impl Comment {
    /// Method for creating a new [`Comment`] from the comment [`String`] and
    /// the [`CommentSpan`]
    ///
    /// # Parameters
    /// - the comment as a [`String`]
    /// - the span of the comment as a [`CommentSpan`]
    #[must_use]
    pub const fn new(comment: String, span: CommentSpan) -> Self {
        Self { comment, span }
    }

    /// Getter method for retrieving the comment content
    #[must_use]
    pub fn comment(&self) -> &str {
        &self.comment
    }

    /// Getter method for retrieving the [`CommentSpan`] of the comment
    #[must_use]
    pub const fn span(&self) -> &CommentSpan {
        &self.span
    }
}

/// Structure for holding all comments found in the document
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

    /// Getter method for retrieving the Vec of [`Comment`]
    #[must_use]
    pub fn comments(&self) -> &[Comment] {
        &self.comments
    }
}
