## coro

An experiment in building a language that's based on coroutines.

The goal is to design and build a "toy" language that implements "asymmetric
coroutines", as described in [1]. Currently, the language implementation is
functional and provides all of the create/resume/yield features.

## Example

```
# A simple generator that yields natural numbers.

def nat = {
  let n = -1;
  while true do {
    let n = n + 1;
    yield n;
  } end
}

let co = create nat

let i = 0
while i < 10 do {
  print (resume co);
  let i = i + 1;
} end
```

## Building and Testing

Use `cargo` to build, run, and test the implementation. There is also a
Makefile for you convenience. There are a few debug features you can pass to
`cargo`:

* `ast` - prints the AST after parsing
* `dbg` - general debugging, prints out result values, coroutine status, etc.
* `instr` - print the compiled linear instructions for each functionta
* `stack` - print the value stack while executing instructions

You can pass these to Cargo like so:

```bash
$ cargo run --features=ast,dbg,instr,stack
```

## References

[1] A. L. D. Moura and R. Ierusalimschy, “Revisiting coroutines,” ACM Trans. Program. Lang. Syst., vol. 31, no. 2, Feb. 2009, issn: 0164-0925. doi: 10.1145/1462166.1462167. [Online]. Available: https://doi.acm.org/10.1145/1462166.1462167.
