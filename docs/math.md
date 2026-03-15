## Range of Unsigned vs Signed Integer types in Rust :

> n = no. of bits
> unsigned: 0 => 2^n - 1
> signed: -2^(n-1) => +2^(n-1) - 1 (2's complement)

- u8: 0 > 255
- i8: -128 > 127
- u16: 0 > 65,535
- i16: -32,768 → 32,767
- u32: 0 → 4,294,967,295
- i32: -2,147,483,648 → 2,147,483,647
- u64: 0 → 18,446,744,073,709,551,615
- i64: -9,223,372,036,854,775,808 → 9,223,372,036,854,775,807
- u128: 0 > 2^(128) - 1
- i128: -2^(127) > 2^(27) - 1
- usize: depends on architecture's pointer width (32 or 64)
- iszie: depends on architecture's pointer width (32 or 64)

## Calculation of KB and KiB values:

