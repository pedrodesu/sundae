// Our own dyn clonable iterator implementation

use dyn_clone::{DynClone, clone_box};

type IteratorItem = ((usize, usize), char);

pub trait DynClonable<'a>: DynClone + Iterator<Item = IteratorItem> + 'a {}

impl<'a, I: Iterator<Item = IteratorItem> + DynClone + 'a> DynClonable<'a> for I {}

pub struct DynClonableIterator<'a>
{
    iter: Box<dyn DynClonable<'a>>,
}

impl<'a> DynClonableIterator<'a>
{
    #[inline]
    pub fn new(iter: Box<dyn DynClonable<'a>>) -> Self
    {
        Self { iter }
    }
}

impl Clone for DynClonableIterator<'_>
{
    #[inline]
    fn clone(&self) -> Self
    {
        Self {
            iter: clone_box(&*self.iter),
        }
    }
}
impl Iterator for DynClonableIterator<'_>
{
    type Item = IteratorItem;

    #[inline]
    fn next(&mut self) -> Option<Self::Item>
    {
        self.iter.next()
    }
}
