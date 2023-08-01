# Chapter 1

```rust
fn id<X>(x: X) -> {
    x
}
```

```multicode
>>>>> rust
fn id<X>(x: X) -> {
    x
}
<<<<<
>>>>> cpp
X id<X>(X x) {
    return x;
}
<<<<<
```
