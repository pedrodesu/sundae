use std::ops::{Range, RangeInclusive};

use miette::SourceSpan;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Span
{
    pub start: usize,
    pub end: usize,
}

impl Span
{
    #[inline]
    pub const fn new(start: usize, end: usize) -> Self
    {
        Self { start, end }
    }

    #[inline]
    pub const fn empty(at: usize) -> Self
    {
        Self::new(at, at)
    }

    #[inline]
    pub const fn single(at: usize) -> Self
    {
        Self::new(at, at + 1)
    }

    #[inline]
    pub const fn inclusive(start: usize, end: usize) -> Self
    {
        Self::new(start, end + 1)
    }

    #[inline]
    pub const fn len(self) -> usize
    {
        self.end - self.start
    }

    #[inline]
    pub const fn range(self) -> Range<usize>
    {
        self.start..self.end
    }

    #[inline]
    pub fn source(self, source: &str) -> &str
    {
        &source[self.range()]
    }
}

impl From<Range<usize>> for Span
{
    #[inline]
    fn from(value: Range<usize>) -> Self
    {
        Self::new(value.start, value.end)
    }
}

impl From<RangeInclusive<usize>> for Span
{
    #[inline]
    fn from(value: RangeInclusive<usize>) -> Self
    {
        Self::inclusive(*value.start(), *value.end())
    }
}

impl From<usize> for Span
{
    #[inline]
    fn from(value: usize) -> Self
    {
        Self::single(value)
    }
}

impl From<Span> for SourceSpan
{
    #[inline]
    fn from(value: Span) -> Self
    {
        (value.start, value.len()).into()
    }
}
