# Permafrost

This crate provides a procedural macro for embedding data into your Rust code.

# Example

```rust, ignore
use permafrost::embed;

/// Expands to a const item that contains a greeting.
macro_rules! greet {
    (
        $who:literal
    ) => {
        embed! {
            #[allow(dead_code)]
            const [< HELLO _ $who:case{upper} >]:concatenate : &str = [< "Hello" " " $who "!" >]:concatenate{string};
        }
    }
}

greet!("Jonas");
```
```rust, ignore
// Recursive expansion of greet! macro
// ====================================

const HELLO_JONAS: &str = r#"Hello Jonas!"#;
```

# Usage

Add the following to your `Cargo.toml`:

```toml
permafrost = "1"
```

# Background

This originally started as a replacement for the `paste` crate, which allows you to concatenate identifiers and apply case transformations to them.

But seeing how the `paste` crate is not regurarly updated anymore, and that I needed case transformations for kebab case, I decided to create a more powerful and flexible alternative.

# Structure

All is passed to the `embed` macro, which is the entry point for the underlying engine.
//!
Initially, only a `token tree` is passed to be top-level transformer in an arbitrary `transformer chain`, then computations are done purely through `token stream`s.

## Blocks

Blocks are the main building blocks of the `embed` macro, as they denote a transformer-capable `token stream`.

Blocks are defined by square brackets, annotated by `<` `>`, thus, `[< (..) >]` denotes a block, where `(..)` is the target `token stream`.
Blocks can also have their own transformer chain, which is to follow the block.

Block syntax is given in EBNF as:

```ebnf
token-stream = token-tree | segment;

token-tree = group | ident | literal | punct | ...;

segment = block transformer-chain;

transformer = ident | ident "{" token-stream "}";

transformer-chain = ":" transformer { "," transformer };

block = "[" "<" token-stream { token-stream } ">" "]" [ transformer-chain ];
```

## Transformers

All currently available transformers are:

| Transformer | Description | Arguments | Example |
| --- | --- | --- | --- |
| `concatenate` | Concatenate the target `token stream` | `ident`, `string`, `r#ident` | `[< (hello [world]):concatenate{ident} >]` |
| `ungroup` | Ungroup the target `token stream` | | `[< (hello [world]):ungroup >]` |
| `flatten` | Flatten the target `token stream` | | `[< (hello [world]):flatten >]` |
| `reverse` | Reverse the target `token stream` | | `[< (hello [world]):reverse >]` |
| `stringify` | Stringify the target `token stream` | | `[< (hello [world]):stringify >]` |
| `unstringify` | Unstringify the target `token stream` | | `[< "hello world":unstringify >]` |
| `case` | Convert the target `token stream` to a specific case | `kebab`, `snake`, `camel`, `pascal`, `upper`, `lower`, `title` | `[< (hello [world]):case{pascal} >]` |
| `count` | Count the number of elements in the target `token stream` | | `[< (hello [world]):count >]` |
| `append` | Append the target `token stream` with another `token stream` | | `[< (hello [world]):append{[!]} >]` |
| `prefix` | Prefix the target `token stream` with another `token stream` | | `[< (hello [world]):prefix{[!]} >]` |

If you believe that a fundamental transformer is missing, please open an issue.