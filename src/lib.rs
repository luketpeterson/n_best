
#![doc = include_str!("../README.md")]

use core::cmp::*;

use std::collections::BinaryHeap;
use ord_by::*;

/// Collection to keep the N highest elements, and discard the others
pub struct NBest<T, F = ()>
    where Self: NBestWrapper<T, F>
{
    heap: BinaryHeap<<Self as NBestWrapper<T, F>>::InnerT>,
    cmp_func: F,
}

/// Internal trait to associate function with [NBestWrapper] struct
#[doc(hidden)]
pub trait NBestWrapper<T, F> {
    type InnerT: Ord;

    fn _wrap_value_(&self, val: T) -> Self::InnerT;
    fn _unwrap_value_(wrapped: Self::InnerT) -> T;
    fn _unwrap_value_ref_<'a>(wrapped: &'a Self::InnerT) -> &'a T;
    fn _cmp_func_() -> fn(&Self, &T, &T) -> Ordering;
}

impl<T: Ord> NBestWrapper<T, ()> for NBest<T, ()> {
    type InnerT = Reverse<T>;

    fn _wrap_value_(&self, val: T) -> Self::InnerT {
        Reverse(val)
    }
    fn _unwrap_value_(wrapped: Self::InnerT) -> T {
        wrapped.0
    }
    fn _unwrap_value_ref_<'a>(wrapped: &'a Self::InnerT) -> &'a T {
        &wrapped.0
    }
    fn _cmp_func_() -> fn(&Self, &T, &T) -> Ordering {
        fn reverse<T: Ord, S: NBestWrapper<T, ()>>(_n_best: &S, a: &T, b: &T) -> Ordering {
            b.cmp(a)
        }
        reverse
    }
}

impl<T, F: Fn(&T, &T) -> Ordering + Clone> NBestWrapper<T, F> for NBest<T, F> {
    type InnerT = Reverse<OrdBy<T, F>>;

    fn _wrap_value_(&self, val: T) -> Self::InnerT {
        Reverse(OrdBy::new(val, self.cmp_func.clone()))
    }
    fn _unwrap_value_(wrapped: Self::InnerT) -> T {
        wrapped.0.into_inner()
    }
    fn _unwrap_value_ref_<'a>(wrapped: &'a Self::InnerT) -> &'a T {
        &wrapped.0.borrow()
    }
    fn _cmp_func_() -> fn(&Self, &T, &T) -> Ordering {
        fn reverse<T, F>(n_best: &NBest<T, F>, a: &T, b: &T) -> Ordering
            where
            NBest<T, F>: NBestWrapper<T, F>,
            F: Fn(&T, &T) -> Ordering
        {
            (n_best.cmp_func)(b, a)
        }
        reverse::<T, F>
    }
}

impl<T: Ord> NBest<T, ()> {
    /// Returns a new NBest with the specified capacity, for items implementing
    /// the [Ord] trait
    pub fn new(n: usize) -> Self {
        Self::new_internal(n, ())
    }

    /// Returns a new NBest, built from the contents of an iterator
    pub fn with_iter<I>(n: usize, iter: I) -> Self
        where I: IntoIterator<Item=T>
    {
        let mut new_collection = Self::new_internal(n, ());
        for val in iter {
            new_collection.push(val);
        }
        new_collection
    }
}

impl<T, F: Fn(&T, &T) -> Ordering + Clone> NBest<T, F> {

    /// Returns a new NBest, using the supplied function to perform comparisons
    pub fn with_cmp_fn(n: usize, func: F) -> Self
        where F: Fn(&T, &T) -> Ordering,
    {
        Self::new_internal(n, func)
    }

    /// Returns a new NBest, built from the contents of an iterator, using the supplied function
    pub fn with_cmp_fn_and_iter<I>(n: usize, func: F, iter: I) -> Self
        where
        F: Fn(&T, &T) -> Ordering,
        I: IntoIterator<Item=T>
    {
        let mut new_collection = Self::new_internal(n, func);
        for val in iter {
            new_collection.push(val);
        }
        new_collection
    }
}

impl<T, F> NBest<T, F>
    where Self: NBestWrapper<T, F>
{
    fn new_internal(n: usize, func: F) -> Self {
        if n < 1 {
            panic!("0 capacity NBest is useless");
        }
        Self {
            heap: BinaryHeap::with_capacity(n),
            cmp_func: func,
        }
    }

    /// Adds a new element, discarding the lowest element if the collection is at capacity
    pub fn push(&mut self, item: T) {
        let value = self._wrap_value_(item);

        //Do we need to evict an existing element?
        if self.len() >= self.capacity() {
            if self.heap.peek().unwrap() > &value { //Order is reversed, so '>' means less-than
                self.heap.pop();
                self.heap.push(value);
            }
        } else {
            self.heap.push(value);
        }
    }

    /// Returns the least element contained within the collection, or None if the collection is empty
    pub fn pop(&mut self) -> Option<T> {
        self.heap.pop().map(|val| Self::_unwrap_value_(val))
    }

    /// Returns the number of elements contained within the collection
    /// 
    /// `len()` will always be less than [capacity](Self::capacity).
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Returns the maximum number of elements the collection will collect
    pub fn capacity(&self) -> usize {
        self.heap.capacity()
    }

    /// Returns an iterator over the elements contained within the collection, in an unspecified order
    pub fn iter(&self) -> impl Iterator<Item=&T> {
        self.heap.iter().map(|val| Self::_unwrap_value_ref_(val))
    }

    /// Empties the collection and returns an iterator over all contained elements, in an unspecified order
    pub fn drain<'a>(&'a mut self) -> impl Iterator<Item=T> + 'a {
        self.heap.drain().map(|val| Self::_unwrap_value_(val))
    }

    /// Consumes the collection, and returns all elements in a sorted [Vec]
    pub fn into_sorted_vec(mut self) -> Vec<T> {
        let mut items: Vec<T> = self.drain().collect();
        let cmp_func = Self::_cmp_func_();
        items.sort_unstable_by(|a, b| cmp_func(&self, a, b));
        items
    }
}

/// Iterator type returned by [IntoIterator::into_iter]
#[doc(hidden)]
pub struct IntoIter<T, F> where NBest<T, F>: NBestWrapper<T, F> {
    iter: core::iter::Map<std::collections::binary_heap::IntoIter<<NBest<T, F> as NBestWrapper<T, F>>::InnerT>, fn(val: <NBest<T, F> as NBestWrapper<T, F>>::InnerT) -> T >
}

impl<T, F> Iterator for IntoIter<T, F>
    where NBest<T, F>: NBestWrapper<T, F>
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }
}

impl<T, F> IntoIterator for NBest<T, F>
    where Self: NBestWrapper<T, F>
{
    type Item = T;
    type IntoIter = IntoIter<T, F>;

    /// Consumes the collection and returns an iterator over the elements, in an unspecified order
    fn into_iter(self) -> Self::IntoIter {
        IntoIter{
            iter: self.heap.into_iter().map(|val| Self::_unwrap_value_(val))
        }
    }
}

impl<T, F> Extend<T> for NBest<T, F>
    where Self: NBestWrapper<T, F>
{
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        for val in iter {
            self.push(val);
        }
    }
}

#[test]
fn test_cmp_fn() {
    let numbers = vec![0, 2, 4, 6, 8, 1, 3, 5, 7, 9];
    let mut n_best = NBest::with_cmp_fn_and_iter(4, |a, b| b.cmp(a), numbers);

    n_best.push(1);
    n_best.push(22);

    assert_eq!(n_best.into_sorted_vec(), vec![0, 1, 1, 2]);
}

#[test]
fn test_ord() {
    let numbers = vec![0, 2, 4, 6, 8, 1, 3, 5, 7, 9];
    let mut n_best = NBest::new(4);
    n_best.extend(numbers);

    n_best.push(1);
    n_best.push(22);

    assert_eq!(n_best.into_sorted_vec(), vec![22, 9, 8, 7]);
}




