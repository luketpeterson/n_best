
# NBest

Convenient collection to gather the N highest elements, and discard the others

This can be useful in the implementation of an algorithm like k-nearest-neighbour, where you
can build an [NBest] using [Iterator::collect]

```rust
use n_best::NBest;

let numbers = vec![9, 2, 4, 6, 8, 1, 3, 5, 7, 0];
let n_best = NBest::with_cmp_fn_and_iter(4, |a, b| b.cmp(a), numbers);

assert_eq!(n_best.into_sorted_vec(), vec![0, 1, 2, 3]);
```

**Future Work:** This implementation uses [BinaryHeap], but an internal implementation would
be more efficient because it currently needs to store a copy of the compare function for every
retained element.

**Future Work:** Explore making the N value a constant parameter.  It might be more efficient, and it would allow the [FromIterator] trait to be implemented.  On the other hand type parameters are uglier to work with than arguments passed at runtime.