# psoftfloat-rs
Software implementation of half-precision floating point numbers according to IEEE 754.
Not all operations are supported.
Only round to nearest, ties to even is implemented.
Note that this implementation does not generate floating-point exception flags for underflow etc.

## WARNING
This implementation is for educational purposes only.
It is explicitly not optimized for speed, but readability.

## Testing

You can execute a couple of special test cases via
```console
$ cd soft_float
$ cargo test
```
This is particularly helpful during development, but doesn't exactly proof correctness of the implementation.

For half-precision floats, we can actually [test them all](https://randomascii.wordpress.com/2014/01/27/theres-only-four-billion-floatsso-test-them-all/).
By default, these exhaustive tests are ignored, but you can execute them via
```console
$ cd soft_float
$ cargo test -- --ignored
```

Furthermore, we use the [Berkeley TestFloat](http://www.jhauser.us/arithmetic/TestFloat.html) programs to verify the implementation.
Exception flags are merely copied over from the `testfloat_gen` output.
You can execute the tests by
```console
$ cd tests
$ make
```

## Resources
- https://en.wikipedia.org/wiki/Half-precision_floating-point_format
- https://float.exposed/
- https://numeral-systems.com/ieee-754-add/
- https://docs.oracle.com/cd/E19957-01/806-3568/ncg_goldberg.html
- https://ciechanow.ski/exposing-floating-point/
- https://www.sci.utah.edu/~beiwang/teaching/cs6210-fall-2016/BonusLecture4.pdf
- https://www.cs.utexas.edu/~byoung/cs429/slides4-fp.pdf
- https://github.com/starkat99/half-rs/blob/main/src/binary16/arch.rs
- https://randomascii.wordpress.com/2014/01/27/theres-only-four-billion-floatsso-test-them-all/
- https://www.netlib.org/fp/
- https://www.eigentales.com/Floating-Point/
- https://pages.cs.wisc.edu/~david/courses/cs552/S12/handouts/guardbits.pdf
- https://github.com/koute/softfloat
- https://github.com/823984418/const_soft_float
- https://github.com/skeeto/scratch/blob/master/misc/float16.c
- https://cs.stackexchange.com/questions/80668/simple-algorithm-for-ieee-754-division-on-8-bit-cpu
- https://link.springer.com/book/10.1007/978-3-319-76526-6
- https://simonv.fr/TypesConvert/?integers
- https://www.sciencedirect.com/book/9781558607989/digital-arithmetic
