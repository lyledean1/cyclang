# Overview


Some high level snippets of the language.


## For Loop
```rust
for (let i = 0; i < 10; i++)
{  
    print(i);
}
```

## Fibonacci Sequence
```rust
fn fib(int n) -> int {
    if (n < 2) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}
print(fib(20));
```

