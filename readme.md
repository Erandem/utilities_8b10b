# `utilities_8b10b`

> A library containing 8b10b encoding/decoding tables, along with a small amount of utility functions.

## Features

### `ufmt`

Adds [`ufmt`](https://crates.io/crates/ufmt) debug formatting to various types, mostly 
for embedded development.

### `avr-progmem`

Implements [`encode_8b10b`] and [`decode_8b10b`], storing the underlying tables
within progmem using [`avr-progmem`](https://crates.io/crates/avr-progmem/0.4.0).