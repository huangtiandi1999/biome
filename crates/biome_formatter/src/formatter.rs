use crate::buffer::BufferSnapshot;
use crate::builders::{FillBuilder, JoinBuilder, JoinNodesBuilder, Line};
use crate::prelude::*;
use crate::{
    Arguments, Buffer, Comments, CstFormatContext, FormatContext, FormatState, FormatStateSnapshot,
    GroupId, VecBuffer,
};

/// Handles the formatting of a CST and stores the context how the CST should be formatted (user preferences).
/// The formatter is passed to the [Format] implementation of every node in the CST so that they
/// can use it to format their children.
pub struct Formatter<'buf, Context> {
    pub(super) buffer: &'buf mut dyn Buffer<Context = Context>,
}

impl<'buf, Context> Formatter<'buf, Context> {
    /// Creates a new context that uses the given formatter context
    pub fn new(buffer: &'buf mut (dyn Buffer<Context = Context> + 'buf)) -> Self {
        Self { buffer }
    }

    /// Returns the format options
    pub fn options(&self) -> &Context::Options
    where
        Context: FormatContext,
    {
        self.context().options()
    }

    /// Returns the Context specifying how to format the current CST
    pub fn context(&self) -> &Context {
        self.state().context()
    }

    /// Returns a mutable reference to the context.
    pub fn context_mut(&mut self) -> &mut Context {
        self.state_mut().context_mut()
    }

    /// Creates a new group id that is unique to this document. The passed debug name is used in the
    /// [std::fmt::Debug] of the document if this is a debug build.
    /// The name is unused for production builds and has no meaning on the equality of two group ids.
    pub fn group_id(&self, debug_name: &'static str) -> GroupId {
        self.state().group_id(debug_name)
    }

    /// Joins multiple [Format] together without any separator
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use biome_formatter::format;
    /// use biome_formatter::prelude::*;
    ///
    /// # fn main() -> FormatResult<()> {
    /// let formatted = format!(SimpleFormatContext::default(), [format_with(|f| {
    ///     f.join()
    ///         .entry(&text("a"))
    ///         .entry(&space())
    ///         .entry(&text("+"))
    ///         .entry(&space())
    ///         .entry(&text("b"))
    ///         .finish()
    /// })])?;
    ///
    /// assert_eq!(
    ///     "a + b",
    ///     formatted.print()?.as_code()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn join<'a>(&'a mut self) -> JoinBuilder<'a, 'buf, (), Context> {
        JoinBuilder::new(self)
    }

    /// Joins the objects by placing the specified separator between every two items.
    ///
    /// ## Examples
    ///
    /// Joining different tokens by separating them with a comma and a space.
    ///
    /// ```
    /// use biome_formatter::{format, format_args};
    /// use biome_formatter::prelude::*;
    ///
    /// # fn main() -> FormatResult<()> {
    /// let formatted = format!(SimpleFormatContext::default(), [format_with(|f| {
    ///     f.join_with(&format_args!(text(","), space()))
    ///         .entry(&text("1"))
    ///         .entry(&text("2"))
    ///         .entry(&text("3"))
    ///         .entry(&text("4"))
    ///         .finish()
    /// })])?;
    ///
    /// assert_eq!(
    ///     "1, 2, 3, 4",
    ///     formatted.print()?.as_code()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn join_with<'a, Joiner>(
        &'a mut self,
        joiner: Joiner,
    ) -> JoinBuilder<'a, 'buf, Joiner, Context>
    where
        Joiner: Format<Context>,
    {
        JoinBuilder::with_separator(self, joiner)
    }

    /// Specialized version of [crate::Formatter::join_with] for joining SyntaxNodes separated by a space, soft
    /// line break or empty line depending on the input file.
    ///
    /// This function inspects the input source and separates consecutive elements with either
    /// a [crate::builders::soft_line_break_or_space] or [crate::builders::empty_line] depending on how many line breaks were
    /// separating the elements in the original file.
    pub fn join_nodes_with_soft_line<'a>(
        &'a mut self,
    ) -> JoinNodesBuilder<'a, 'buf, Line, Context> {
        JoinNodesBuilder::new(soft_line_break_or_space(), self)
    }

    /// Specialized version of [crate::Formatter::join_with] for joining SyntaxNodes separated by one or more
    /// line breaks depending on the input file.
    ///
    /// This function inspects the input source and separates consecutive elements with either
    /// a [crate::builders::hard_line_break] or [crate::builders::empty_line] depending on how many line breaks were separating the
    /// elements in the original file.
    pub fn join_nodes_with_hardline<'a>(&'a mut self) -> JoinNodesBuilder<'a, 'buf, Line, Context> {
        JoinNodesBuilder::new(hard_line_break(), self)
    }

    /// Specialized version of [crate::Formatter::join_with] for joining SyntaxNodes separated by a simple space.
    ///
    /// This function *disregards* the input source and always separates consecutive elements with a plain
    /// [crate::builders::space], forcing a flat layout regardless of any line breaks or spaces were separating
    /// the elements in the original file.
    ///
    /// This function should likely only be used in a `best_fitting!` context, where one variant attempts to
    /// force a list of nodes onto a single line without any possible breaks, then falls back to a broken
    /// out variant if the content does not fit.
    pub fn join_nodes_with_space<'a>(&'a mut self) -> JoinNodesBuilder<'a, 'buf, Space, Context> {
        JoinNodesBuilder::new(space(), self)
    }

    /// Concatenates a list of [crate::Format] objects with spaces and line breaks to fit
    /// them on as few lines as possible. Each element introduces a conceptual group. The printer
    /// first tries to print the item in flat mode but then prints it in expanded mode if it doesn't fit.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use biome_formatter::prelude::*;
    /// use biome_formatter::{format, format_args};
    ///
    /// # fn main() -> FormatResult<()> {
    /// let formatted = format!(SimpleFormatContext::default(), [format_with(|f| {
    ///     f.fill()
    ///         .entry(&soft_line_break_or_space(), &text("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"))
    ///         .entry(&soft_line_break_or_space(), &text("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"))
    ///         .entry(&soft_line_break_or_space(), &text("cccccccccccccccccccccccccccccc"))
    ///         .entry(&soft_line_break_or_space(), &text("dddddddddddddddddddddddddddddd"))
    ///         .finish()
    /// })])?;
    ///
    /// assert_eq!(
    ///     "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\ncccccccccccccccccccccccccccccc dddddddddddddddddddddddddddddd",
    ///     formatted.print()?.as_code()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```rust
    /// use biome_formatter::prelude::*;
    /// use biome_formatter::{format, format_args};
    ///
    /// # fn main() -> FormatResult<()> {
    /// let entries = vec![
    ///     text("<b>Important: </b>"),
    ///     text("Please do not commit memory bugs such as segfaults, buffer overflows, etc. otherwise you "),
    ///     text("<em>will</em>"),
    ///     text(" be reprimanded")
    /// ];
    ///
    /// let formatted = format!(SimpleFormatContext::default(), [format_with(|f| {
    ///     f.fill().entries(&soft_line_break(), entries.iter()).finish()
    /// })])?;
    ///
    /// assert_eq!(
    ///     &std::format!("<b>Important: </b>\nPlease do not commit memory bugs such as segfaults, buffer overflows, etc. otherwise you \n<em>will</em> be reprimanded"),
    ///     formatted.print()?.as_code()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn fill<'a>(&'a mut self) -> FillBuilder<'a, 'buf, Context> {
        FillBuilder::new(self)
    }

    /// Formats `content` into an interned element without writing it to the formatter's buffer.
    pub fn intern(&mut self, content: &dyn Format<Context>) -> FormatResult<Option<FormatElement>> {
        let mut buffer = VecBuffer::new(self.state_mut());
        crate::write!(&mut buffer, [content])?;
        let elements = buffer.into_vec();

        Ok(self.intern_vec(elements))
    }

    pub fn intern_vec(&mut self, mut elements: Vec<FormatElement>) -> Option<FormatElement> {
        match elements.len() {
            0 => None,
            // Doesn't get cheaper than calling clone, use the element directly
            // SAFETY: Safe because of the `len == 1` check in the match arm.
            1 => Some(elements.pop().unwrap()),
            _ => Some(FormatElement::Interned(Interned::new(elements))),
        }
    }
}

impl<Context> Formatter<'_, Context>
where
    Context: FormatContext,
{
    /// Take a snapshot of the state of the formatter
    #[inline]
    pub fn state_snapshot(&self) -> FormatterSnapshot {
        FormatterSnapshot {
            buffer: self.buffer.snapshot(),
            state: self.state().snapshot(),
        }
    }

    #[inline]
    /// Restore the state of the formatter to a previous snapshot
    pub fn restore_state_snapshot(&mut self, snapshot: FormatterSnapshot) {
        self.state_mut().restore_snapshot(snapshot.state);
        self.buffer.restore_snapshot(snapshot.buffer);
    }
}

impl<Context> Formatter<'_, Context>
where
    Context: CstFormatContext,
{
    /// Returns the comments from the context.
    pub fn comments(&self) -> &Comments<Context::Language> {
        self.context().comments()
    }
}

impl<Context> Buffer for Formatter<'_, Context> {
    type Context = Context;

    #[inline(always)]
    fn write_element(&mut self, element: FormatElement) -> FormatResult<()> {
        self.buffer.write_element(element)
    }

    fn elements(&self) -> &[FormatElement] {
        self.buffer.elements()
    }

    #[inline(always)]
    fn write_fmt(&mut self, arguments: Arguments<Self::Context>) -> FormatResult<()> {
        for argument in arguments.items() {
            argument.format(self)?;
        }
        Ok(())
    }

    fn state(&self) -> &FormatState<Self::Context> {
        self.buffer.state()
    }

    fn state_mut(&mut self) -> &mut FormatState<Self::Context> {
        self.buffer.state_mut()
    }

    fn snapshot(&self) -> BufferSnapshot {
        self.buffer.snapshot()
    }

    fn restore_snapshot(&mut self, snapshot: BufferSnapshot) {
        self.buffer.restore_snapshot(snapshot)
    }
}

/// Snapshot of the formatter state used to handle backtracking if
/// errors are encountered in the formatting process and the formatter
/// has to fall back to printing raw tokens
///
/// In practice this only saves the set of printed tokens in debug
/// mode and compiles to nothing in release mode
pub struct FormatterSnapshot {
    buffer: BufferSnapshot,
    state: FormatStateSnapshot,
}
