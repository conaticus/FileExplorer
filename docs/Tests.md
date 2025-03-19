# Rules and guide for writing tests

Tests should be written for every function.
Every Test should be in a ``mod test`` block with ``#[cfg(test)]`` -> This ensures that the tests
are only build when the ``cargo test`` command is executed in the ``src-tauri`` folder

## Code example

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }
}
```

### Different assertions

````rust
#[test]
fn test_assertions() {
    assert!(true);
    assert_eq!(5, 5);
    assert_ne!(5, 3);
}
````

### Panic when there is something major wrong

````rust
pub fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("Cannot divide by zero!");
    }
    a / b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Cannot divide by zero!")]
    fn test_divide_by_zero() {
        divide(10, 0);
    }
}
````

### Flexible use of Result

````rust
#[test]
fn test_with_result() -> Result<(), String> {
    if 2 + 2 == 4 {
        Ok(())
    } else {
        Err("Math is broken!".to_string())
    }
}
````

# IT's -> Integration Tests

Integration Tests are written from an outside perspective and are in a seperate folder in ``tesst``
There you need to import the module you want to test and test it like other parts of the code see
it. 

```rust
// tests/math_test.rs
use your_project::multiply;

#[test]
fn test_multiplication() {
    assert_eq!(multiply(3, 4), 12);
    assert_eq!(multiply(-2, 5), -10);
}
```