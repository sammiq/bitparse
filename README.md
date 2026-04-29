# BitParse

Can we have [bitwise](https://github.com/mellowcandle/bitwise)?

No, we have bitwise at home.

## Reasoning

This is an experiment to teach myself some parsing as well as add some functionality i wanted
when i was using bitwise.

It differs from bitwise in that its far less pretty, ditches interactive mode, but does a few additional operations:

1. It will output the result of any expression as single and double precision number in addition to other representations.

2. It will accept single (1.23f) and double (1.23) precision numbers in the expression, these
will be converted to bits.

2. It adds a bit unpacker (-u / --unpack) which takes a list of offsets and unpacks the values
at those bit offsets.

It also changes the -w/--width flag to something more understandable (IMO), d for double and q for quad.

