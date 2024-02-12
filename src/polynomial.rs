use crate::ops;
use rand::Rng;
use zeroize::Zeroize;

/// A struct representing a polynomial with coefficients in `GF(2^8)`.
///
/// The index of each element in the `coefficients` vector represents the power of the corresponding term.
/// For instance, a polynomial `ax^2 + bx + c` is represented as `coefficients: vec![c, b, a]`.
///
pub struct Polynomial {
    /// The coefficients of the polynomial, ordered from the `intercept`
    /// up to the highest-degree term.
    coefficients: Vec<u8>,
}

impl Polynomial {
    /// Creates a new polynomial with a specified `intercept` and randomly generated coefficients.
    ///
    /// ## Arguments
    /// * `intercept` - The constant term of the polynomial.
    /// * `degree` - The highest power of `x` that appears in the polynomial.
    ///
    /// ## Returns
    /// * The newly created Polynomial.
    pub(crate) fn generate(intercept: u8, degree: u8) -> Polynomial {
        // Initialize the coefficients vector with zeros
        // in the size of the `degree`, plus 1 additional byte for the `intercept`.
        let mut coefficients = vec![0u8; (degree + 1) as usize];

        // Assign the constant-term (`intercept`) to the provided input.
        coefficients[0] = intercept;
        // Randomly generate the remaining coefficients.
        rand::thread_rng().fill(&mut coefficients[1..]);

        Polynomial { coefficients }
    }

    /// Evaluates the polynomial at a given point `x` using Horner's method.
    ///
    /// ## Arguments
    /// * `x` - The point at which to evaluate the polynomial.
    ///
    /// ## Returns
    /// * The value of the polynomial at `x`.
    ///
    /// ## Panics
    /// * If `x` is zero, since the evaluation at `x = 0` is not allowed.
    /// This is a safeguard to prevent revealing the secret byte set as the constant term.
    pub(crate) fn evaluate(&self, x: u8) -> u8 {
        // Mathematically, evaluating a polynomial at `x = 0` is valid and results to the constant term (`self.coefficients[0]`).
        // However, that's not allowed in order to prevent revealing the secret byte, which in this case is the constant term.
        //
        // The `evaluate` function is crate-level, not public; this safe=guard is implemented to prevent incorrect third-party implementations
        // or changes in the code that could lead to accidental exposure of the secret bytes.
        //
        // Normally invoked from `split` with x-coordinates in the range of `1..=255`, therefor this should never occur.
        if x == 0 {
            panic!("evaluation not allowed for x = 0");
        }

        // Start from the highest degree coefficient.
        // Coefficients are guaranteed to have at least one element,
        // thus the `expect` method will never cause a runtime error in a correct implementation.
        let mut result = self.coefficients.last().copied().expect("empty coefficients");

        // Iterate over the coefficients in reverse
        for coefficient in self.coefficients.iter().rev().skip(1) {
            // Horner's method for polynomial evaluation.
            result = ops::add(ops::mult(result, x), *coefficient);
        }
        result
    }

    /// Computes the value of a polynomial at a given point `x` using Lagrange interpolation.
    ///
    /// ## Arguments
    /// * `x_samples` - Array of x-coordinates of the dataset.
    /// * `y_samples` - Array of y-coordinates of the dataset, each corresponding to `x_samples`.
    /// * `x` - The x-coordinate at which the interpolated polynomial is to be computed.
    ///
    /// ## Returns
    /// * The interpolated value of the polynomial at `x`.
    ///
    /// ## Notes
    /// * This function assumes that `x_samples` and `y_samples` have the same length and contain no duplicate x-values.
    /// The caller must ensure this for performance reasons in order to avoid reduntant checks when iterating.
    pub(crate) fn interpolate(x_samples: &[u8], y_samples: &[u8], x: u8) -> u8 {
        let limit = x_samples.len();
        let mut result = 0;
        // Iterate over each sample to construct the Lagrange basis polynomial.
        for i in 0..limit {
            let mut basis = 1;
            // Construct the basis polynomial for each i-th term.
            for j in 0..limit {
                if i == j {
                    continue;
                }
                // Calculate the numerator and denominator for the Lagrange basis.
                let num = ops::add(x, x_samples[j]);
                let denom = ops::add(x_samples[i], x_samples[j]);
                let term = ops::div(num, denom);
                // Multiply the basis by the current term.
                basis = ops::mult(basis, term);
            }
            // Multiply the y-sample by the basis and add to the result.
            let group = ops::mult(y_samples[i], basis);
            result = ops::add(result, group);
        }
        result
    }
}

// This is important for security purposes to prevent sensitive data
// from staying in memory after the Polynomial is no longer required and dropped.
impl Drop for Polynomial {
    fn drop(&mut self) {
        // Clear memory associated with the coefficients.
        self.coefficients.zeroize();
    }
}

// Test cases for the operations of the Polynomial struct.
#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    // Polynomial generation with random coefficients.
    #[test]
    fn it_generates() -> Result<(), Box<dyn Error>> {
        let degree = 3;
        let intercept = 5;
        let polynomial = Polynomial::generate(intercept, degree);

        // The first coefficient is the intercept
        assert_eq!(polynomial.coefficients[0], intercept);
        // The number of coefficients is equal to the degree plus the intercept
        assert_eq!(polynomial.coefficients.len(), (degree + 1) as usize);
        Ok(())
    }

    // Polynomial interpolation with known samples.
    #[test]
    fn it_interpolates() {
        // Assign points for interpolation.
        let x_samples = [0x3D, 0xA7, 0x1E];
        let y_samples = [0x1A, 0x2B, 0x4C];
        // Set a specific point.
        let x = 0x5A;

        assert_eq!(Polynomial::interpolate(&x_samples, &y_samples, x), 0xCE);
    }

    // Polynomial evaluation with known coefficients.
    #[test]
    fn it_evaluates() {
        // Assign coefficients for the polynomial.
        let coefficients = vec![0x7C, 0x3E, 0x4F, 0x2A, 0x07];
        // Create a polynomial with the coefficients.
        let polynomial = Polynomial { coefficients };
        // Set a specific point.
        let x = 0x2A;

        assert_eq!(polynomial.evaluate(x), 0xEF);
    }

    // Polynomial evaluation with known coefficients.
    #[test]
    #[should_panic(expected = "evaluation not allowed for x = 0")]
    fn it_fails_to_evaluate_zero() {
        // Assign coefficients for the polynomial.
        let coefficients = vec![0x7C, 0x3E, 0x4F, 0x2A, 0x07];
        // Create a polynomial with the coefficients.
        let polynomial = Polynomial { coefficients };
        // Set a specific point.
        let x = 0x00;

        polynomial.evaluate(x);
    }
}
