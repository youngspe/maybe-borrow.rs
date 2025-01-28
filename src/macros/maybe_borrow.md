As a hypothetical demonstration:

```rust ignore
fn get_data_or_initialize(mut borrow: &mut SomeType) -> BorrowedData<'x> {
    maybe_borrow!(for<'x> |borrow| -> BorrowedData<'x> {
        let data: BorrowedData = borrow.get_borrowed_data_mut();

        // Return the borrowed data only if it's initialized
        if (data.is_initialized()) {
            return_borrowed!(data);
        }
    });

    // `borrow` is still accessible because its data did not escape the above
    // block.
    borrow.initialize();
    borrow.get_borrowed_data_mut()
}
```

## Control flow

This macro internally places `$block` inside a closure, so returning or breaking from within will not work as expected.

- The <dfn>[`return_borrowed!`]</dfn> macro is used to return from the function that contains the `maybe_borrow!` invocation.
- If `return_borrowed!` is not used and the block exits normally, it will evaluate to the trailing expression like any block. The trailing expression cannot reference `$ptr`.
- If the macro completes without returning, the variable referenced by `$ptr` is still fully accessible because no borrowed data was allowed to escape the block.

## Examples

### Conditionally returning a mutable reference

```rust
use std::{borrow::Borrow, collections::HashMap};
use maybe_borrow::maybe_borrow;

/// Finds the first key in `keys` that can be found in `map` and returns a
/// mutable reference to its value, or `None` if none of the keys exist in
/// `map`.
fn get_first_available_mut<T>(
    mut map: &mut HashMap<String, T>,
    keys: impl IntoIterator<Item: Borrow<str>>,
) -> Option<&mut T> {
    for key in keys {
        maybe_borrow!(for<'x> |map| -> Option<&'x mut T> {
            if let value @ Some(_) = map.get_mut(key.borrow()) {
                // Use `return_borrowed!()` to return with borrowed data.
                return_borrowed!(value);
            }
        });
    }

    None
}

let mut map = HashMap::from_iter([
    ("a".into(), 1),
    ("c".into(), 2),
    ("d".into(), 3),
    ("f".into(), 4),
]);

let x = get_first_available_mut(&mut map, ["b", "d", "e", "f"]).unwrap();

*x += 100;

assert_eq!(map["d"], 103);
```

### Working with multiple lifetimes

If multiple lifetimes are supplied, each argument will be associated with its respective lifetime.
If there are more arguments than lifetimes, the remaining arguments will be associated with the
last lifetime in the list.

```rust
use std::{borrow::Borrow, collections::HashMap};
use maybe_borrow::maybe_borrow;

/// Finds the first key in `keys` that can be found in both maps and returns a
/// pair of mutable references to their values, or `None` if none of the keys
/// exist in both maps.
fn get_first_available_pair_mut<'a, 'b, T, U>(
    mut map_a: &'a mut HashMap<String, T>,
    mut map_b: &'b mut HashMap<String, U>,
    keys: impl IntoIterator<Item: Borrow<str>>,
) -> Option<(&'a mut T, &'b mut U)> {
    for key in keys {
        maybe_borrow!(for<'x, 'y> |
            map_a,
            map_b,
        | -> Option<(&'x mut T, &'y mut U)> {
            let pair = map_a
                .get_mut(key.borrow())
                .and_then(|a| Some((a, map_b.get_mut(key.borrow())?)));


            if pair.is_some() {
                return_borrowed!(pair);
            }
        });
    }

    None
}

let mut map_a = HashMap::from_iter([
    ("a".into(), 1),
    ("c".into(), 2),
    ("d".into(), 3),
    ("f".into(), 4),
]);

let mut map_b = HashMap::from_iter([
    ("b".into(), 1),
    ("d".into(), 2),
    ("e".into(), 3),
]);

let (a, b) = get_first_available_pair_mut(
    &mut map_a,
    &mut map_b,
    ["a", "b", "c", "d", "e"],
).unwrap();

*a += 100;
*b += 100;

assert_eq!(map_a["d"], 103);
assert_eq!(map_b["d"], 102);
```
