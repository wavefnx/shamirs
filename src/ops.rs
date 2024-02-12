use subtle::{ConditionallySelectable, ConstantTimeEq};

/// The irreducible polynomial in `GF(2^8)`.
const IRREDUCIBLE_POLYNOMIAL: u8 = 0x1B;

/// Division of two `u8` values in `GF(2^8)` utilizing constant-time operations.
/// In a Galois Field `a / b = a * b^-1` holds true.
///
/// ## Arguments
/// * `a` - Dividend.
/// * `b` - Divisor.
///
/// ## Returns
/// * Result of division if `b` is non-zero.
///
/// ## Panics
/// * If `b` is zero, since
pub(crate) fn div(a: u8, b: u8) -> u8 {
    if b == 0 {
        // As a conditional, it may provide side-channel (timing) based information,
        // although this should never occur and it`s just a safe-guard.
        panic!("division by zero");
    }

    // Applying the formula `a * b^-1`
    let result = mult(a, inverse(b));

    // By using the constant-time conditional select operation, the execution time of the function
    // does not depend on the value of the `choice` parameter. This prevents the exploitations of timing differences
    // to gain information about the operands, thus mitigating the risk of side-channel (timing) attacks.
    //
    // Check in constant-time if `a` is *not* equal to zero.
    let choice = a.ct_ne(&0);
    //
    // The function below is equivalent to:
    // if a != 0 { result } else { 0 };
    // while avoiding branching, which may impact the execution time.
    u8::conditional_select(&a, &result, choice)
}

/// Performs multiplication of two `u8` values in `GF(2^8)`.
///
/// ## Arguments
/// * `a` - Multiplicand.
/// * `b` - Multiplier.
///
/// ## Returns
/// * Result of the multiplication.
///
// It's important to prevent any optimisations (`jne`, `je`, ...)
// that can introduce branching, impact the constant execution time
// and provide side-channel (timing) based information.
//
// The compiler would naturally try to optimize the code
// and instead of the calling function invoking `mult` each time,
// it would embed the `mult` function into it.
//
// The `#[inline(never)]` attribute prevents that from happening.
#[inline(never)]
pub(crate) fn mult(a: u8, b: u8) -> u8 {
    let mut product = 0u8;

    for i in (0..8).rev() {
        // Extract the bit from `b` at the current position.
        // If the bit is set, `a` will be XORed with `product`,
        // otherwise the product remains unchanged.
        let contribution = (b >> i) & 1;

        // If the MSB of `product` is set, the polynomial
        // will be reduced using the irreducible polynomial `0x1B`
        // later in the loop.
        let reduction = (product >> 7) & 1;

        // The product is updated as follows:
        // 1. Double the `product` using a left bit-shift, moving each bit one position to the left.
        // 2. XOR with the `contribution`.
        // 3. XOR with `reduction` if polynomial reduction is necessary.
        product = (product << 1) ^ (contribution * a) ^ (reduction * IRREDUCIBLE_POLYNOMIAL);
    }

    product
}

/// Performs addition (`XOR`) of two `u8` values in `GF(2^8)`.
///
/// ## Arguments
/// * `a` - First operand.
/// * `b` - Second operand.
///
/// ## Returns
/// * Result of the `XOR` operation.
pub(crate) fn add(a: u8, b: u8) -> u8 {
    // Addition in a finite field is equivalent to the `XOR` operation.
    a ^ b
}

/// Computes the multiplicative inverse of a value in `GF(2^8)`.
///
/// The multiplicative inverse of `a` in `GF(2^8)` can be expressed as `a^254 = a^-1`,
/// since the multiplicative order of `GF(2^8)` is `255`, excluding zero.
///
/// ## Arguments
/// * `a` - The value to find the inverse of.
///
/// ## Returns
/// * The multiplicative inverse.
///
/// ## Panics
/// * If `a` is zero, since the inverse of zero is undefined.
fn inverse(a: u8) -> u8 {
    if a == 0 {
        // As a conditional, it may provide side-channel (timing) based information,
        // although the inverse of zero is undefined therefor it`s just a safe-guard
        // and should never occur.
        panic!("inverse of zero is undefined");
    }

    // initialization: b = a -> a^1
    let mut b = a;

    for _ in 0..6 {
        // 1st iteration: b = a^2 then b= a^3
        // 2nd iteration: b = a^6 then b= a^7
        // 3rd iteration: b = a^14 then b= a^15
        // 4th iteration: b = a^30 then b= a^31
        // 5th iteration: b = a^62 then b= a^63
        // 6th iteration: b = a^126 then b= a^127
        b = mult(b, b);
        b = mult(b, a);
    }

    // finalization: b = a^254 -> a^-1
    mult(b, b)
}

// Tests for basic arithmetic operation in `GF(2^8)`.
#[cfg(test)]
mod tests {
    use super::*;
    // Tests for the `add` function.
    #[test]
    fn it_adds() {
        // With zero.
        assert_eq!(add(0x00, 0x00), 0x00);
        // Same values (should result in zero due to the nature of `XOR`).
        assert_eq!(add(0x01, 0x01), 0x00);
        assert_eq!(add(0x0F, 0x0F), 0x00);
        // A maximum value and a non-zero value.
        assert_eq!(add(0xFF, 0x01), 0xFE);
        // Additional test cases for various non-zero values
        assert_eq!(add(0x0A, 0x0B), 0x01);
        assert_eq!(add(0x0F, 0x01), 0x0E);
        assert_eq!(add(0x26, 0x81), 0xA7);
        assert_eq!(add(0x7B, 0xC6), 0xBD);
    }
    // Tests for the `mult` function.
    #[test]
    fn it_mults() {
        // With zero.
        assert_eq!(mult(0x00, 0x00), 0x00);
        // With one (identity element).
        assert_eq!(mult(0x01, 0x01), 0x01);
        // A maximum value and a non-one value.
        assert_eq!(mult(0xFF, 0x02), 0xE5);
        // Additional test cases for various non-zero values
        assert_eq!(mult(0x8C, 0x2A), 0x3F);
        assert_eq!(mult(0xB4, 0xD9), 0xEB);
        assert_eq!(mult(0x52, 0xF2), 0x94);
        assert_eq!(mult(0x12, 0xAA), 0x01);
    }
    // Tests for the `div` function.
    #[test]
    fn it_divs() {
        // // With one.
        assert_eq!(div(0x01, 0x01), 0x01);
        // A maximum value and a non-one divisor.
        assert_eq!(div(0xFF, 0x03), 0x55);
        // Division by the same number.
        assert_eq!(div(0xF3, 0xF3), 0x01);
        assert_eq!(div(0xC1, 0xC1), 0x01);
        assert_eq!(div(0xA7, 0xA7), 0x01);
        // Additional test cases for various non-zero values
        assert_eq!(div(0x06, 0x02), 0x03);
        assert_eq!(div(0x19, 0x5D), 0x8F);
        assert_eq!(div(0x7B, 0x3A), 0xF9);
        assert_eq!(div(0x8C, 0x2A), 0xD4);
    }
    // Test and fail dividing by zero with the `div` function.
    #[test]
    #[should_panic(expected = "division by zero")]
    fn it_fails_at_div_zero() {
        div(0xFF, 0x00);
    }
    // Tests for the `inverse` function.
    #[test]
    fn it_inverts() {
        // The inverse of one (should be one, as it`s the identity element).
        assert_eq!(inverse(0x01), 0x01);
        // The inverse of a non-one value.
        assert_eq!(inverse(0x02), 0x8D);
        // The inverse of various non-zero values
        assert_eq!(inverse(0x03), 0xF6);
        assert_eq!(inverse(0x10), 0x74);
        assert_eq!(inverse(0x53), 0xCA);
        assert_eq!(inverse(0xB7), 0x71);
        assert_eq!(inverse(0xFF), 0x1C);
    }
    // Test and fail finding inverse of zero as it`s undefined.
    #[test]
    #[should_panic(expected = "inverse of zero is undefined")]
    fn it_fails_at_inverse_zero() {
        inverse(0x00);
    }
}
