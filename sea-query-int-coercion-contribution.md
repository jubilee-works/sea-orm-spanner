# sea-query contribution: lenient integer `ValueType::try_from`

Goal: let a typed integer (`i8`/`i16`/`i32`/`i64`/`u8`/`u16`/`u32`/`u64`) be read
from **any** integer `Value` variant via a *checked* conversion, instead of only
its own exact variant.

This is what `sea-orm-spanner` needs: Spanner has a single integer type
(`INT64`), so the proxy backend can only emit `Value::BigInt`. Today a model
field declared as `i32` cannot be read from it, because
`i32::try_from(Value::BigInt(..))` returns `Err`.

Repo: `https://github.com/SeaQL/sea-query` — file `src/value.rs`.

---

## 1. Problem

`ValueType::try_from` is strict: each integer type accepts only its own `Value`
variant. Verified matrix (sea-query 1.0.1):

| read as | `Value::Int` | `Value::BigInt` |
|---|---|---|
| `i32` | `Ok` | `Err(ValueTypeErr)` |
| `i64` | `Err(ValueTypeErr)` | `Ok` |

The mock and proxy backends materialise values as `Value` and read them back
through `ValueType::try_from` (sqlx-backed backends decode through sqlx, so they
are unaffected). When the underlying database has fewer integer widths than Rust
(Spanner: only `INT64`), a backend can emit only one variant, so exactly one of
`i32`/`i64` is always unreadable. There is no model-type information at the
backend layer to pick the "right" variant.

## 2. Root cause

`src/value.rs`, the `type_to_value!` macro (line ~1097). Its `ValueType` impl:

```rust
fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
    match v {
        Value::$name(Some(x)) => Ok(x),
        _ => Err(ValueTypeErr),
    }
}
```

All 8 integer types are generated through this macro (lines ~1139-1146):

```rust
type_to_value!(i8,  TinyInt,       TinyInteger);
type_to_value!(i16, SmallInt,      SmallInteger);
type_to_value!(i32, Int,           Integer);
type_to_value!(i64, BigInt,        BigInteger);
type_to_value!(u8,  TinyUnsigned,  TinyUnsigned);
type_to_value!(u16, SmallUnsigned, SmallUnsigned);
type_to_value!(u32, Unsigned,      Unsigned);
type_to_value!(u64, BigUnsigned,   BigUnsigned);
```

## 3. Proposed change

Add a dedicated `int_type_to_value!` macro for the integer types. It is identical
to `type_to_value!` except its `try_from` accepts **any** integer `Value` variant
and converts with `<$type>::try_from(..)` (the std checked integer conversions),
returning `ValueTypeErr` when the value does not fit the target type.

Add this macro next to `type_to_value!` in `src/value.rs`:

```rust
macro_rules! int_type_to_value {
    ( $type: ty, $name: ident, $col_type: expr ) => {
        impl From<$type> for Value {
            fn from(x: $type) -> Value {
                Value::$name(Some(x))
            }
        }

        impl Nullable for $type {
            fn null() -> Value {
                Value::$name(None)
            }
        }

        impl ValueType for $type {
            fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
                // Accept any integer variant, converting with a *checked*
                // std conversion. NULL and out-of-range both map to ValueTypeErr,
                // preserving the original strict behaviour for those cases.
                let converted: Option<$type> = match v {
                    Value::TinyInt(x)       => x.and_then(|n| <$type>::try_from(n).ok()),
                    Value::SmallInt(x)      => x.and_then(|n| <$type>::try_from(n).ok()),
                    Value::Int(x)           => x.and_then(|n| <$type>::try_from(n).ok()),
                    Value::BigInt(x)        => x.and_then(|n| <$type>::try_from(n).ok()),
                    Value::TinyUnsigned(x)  => x.and_then(|n| <$type>::try_from(n).ok()),
                    Value::SmallUnsigned(x) => x.and_then(|n| <$type>::try_from(n).ok()),
                    Value::Unsigned(x)      => x.and_then(|n| <$type>::try_from(n).ok()),
                    Value::BigUnsigned(x)   => x.and_then(|n| <$type>::try_from(n).ok()),
                    _ => return Err(ValueTypeErr),
                };
                converted.ok_or(ValueTypeErr)
            }

            fn type_name() -> String {
                stringify!($type).to_owned()
            }

            fn array_type() -> ArrayType {
                ArrayType::$name
            }

            fn column_type() -> ColumnType {
                use ColumnType::*;
                $col_type
            }
        }
    };
}
```

Then switch the 8 integer calls from `type_to_value!` to `int_type_to_value!`:

```rust
int_type_to_value!(i8,  TinyInt,       TinyInteger);
int_type_to_value!(i16, SmallInt,      SmallInteger);
int_type_to_value!(i32, Int,           Integer);
int_type_to_value!(i64, BigInt,        BigInteger);
int_type_to_value!(u8,  TinyUnsigned,  TinyUnsigned);
int_type_to_value!(u16, SmallUnsigned, SmallUnsigned);
int_type_to_value!(u32, Unsigned,      Unsigned);
int_type_to_value!(u64, BigUnsigned,   BigUnsigned);
```

Nothing else changes: `From`, `Nullable`, `type_name`, `array_type`, and
`column_type` are byte-for-byte identical to the original macro, so writes,
binding, NULL handling, and DDL are untouched. Only the *read* (`try_from`) is
made lenient.

### Why `<$type>::try_from`

The std library provides `TryFrom` between every pair of integer types, and the
reflexive case (`i32: TryFrom<i32>`, error `Infallible`) via the blanket
`TryFrom<U> for T where U: Into<T>`. So `<$type>::try_from(n)` compiles for every
source/target combination, and `.ok()` normalises the differing error types to a
single `Option<$type>` so the match arms unify.

## 4. Tests

Add to `src/value.rs`'s test module (`tests/value` / `mod tests`):

```rust
#[test]
fn integer_value_type_is_lenient_and_checked() {
    use crate::{Value, ValueType};

    // Widen / narrow within range.
    assert_eq!(<i32 as ValueType>::try_from(Value::BigInt(Some(123))).unwrap(), 123);
    assert_eq!(<i64 as ValueType>::try_from(Value::Int(Some(25))).unwrap(), 25);
    assert_eq!(<i8 as ValueType>::try_from(Value::BigInt(Some(7))).unwrap(), 7);

    // Out of range -> Err (no silent truncation).
    assert!(<i32 as ValueType>::try_from(Value::BigInt(Some(i64::MAX))).is_err());
    assert!(<u32 as ValueType>::try_from(Value::Int(Some(-1))).is_err());

    // NULL -> Err, preserving original semantics.
    assert!(<i32 as ValueType>::try_from(Value::Int(None)).is_err());

    // Non-integer -> Err.
    assert!(<i32 as ValueType>::try_from(Value::String(None)).is_err());
}
```

(Verified independently: all of the above hold for the proposed macro.)

## 5. Design notes / tradeoffs to raise in the PR

- **Behaviour change, not a break of the public API.** Conversions that used to
  return `Err` (e.g. `i32::try_from(Value::BigInt(..))`) now succeed when the
  value fits. Code that *relied on* the strict rejection — most plausibly some
  mock-backend tests asserting an exact variant mismatch — would change. This is
  worth calling out explicitly to the maintainers.
- **No silent data loss.** Conversion is checked; out-of-range yields
  `ValueTypeErr`, same as the previous strict failure.
- **Signed/unsigned crossing** is allowed but range-checked (e.g. a negative
  value read as `u32` is `Err`). If the maintainers prefer to be conservative,
  the match can be narrowed to same-signedness variants only; the Spanner use
  case only needs signed widening/narrowing (`BigInt` ↔ `i32`).
- **Scope.** Only `ValueType::try_from` for the 8 integer scalar types changes.
  Array value types, `From`, `Nullable`, and column-type mapping are untouched.

## 6. Alternative: fix it in sea-orm instead (smaller blast radius)

If SeaQL prefers not to broaden `ValueType` globally, the same outcome can be
achieved in **sea-orm** by coercing only inside the proxy read path, leaving
sea-query strict. In `sea-orm/src/database/proxy.rs`, `ProxyRow::try_get<T>`
already knows the target via `T::array_type()`; it can normalise an integer
`Value` toward that target before calling `T::try_from`. That scopes the change
to the proxy backend (the one backend that materialises `Value`s and needs the
coercion) and does not touch sqlx/mock paths.

The sea-query change is simpler and benefits every consumer of `Value`
conversions; the sea-orm change is more surgical. Either resolves the
`sea-orm-spanner` `i32`-over-`INT64` failure.
