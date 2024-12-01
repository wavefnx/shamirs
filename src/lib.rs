#![forbid(unsafe_code)]
#![warn(clippy::all)]

mod ops;
mod polynomial;
use polynomial::Polynomial;
/// Splits a secret into multiple shares.
///
/// ## Arguments
/// * `secret` - The secret to be split.
/// * `threshold` - Minimum number of shares required to reconstruct the secret.
/// * `parts` - Total number of shares to create.
///
/// ## Returns
/// * A vector of shares if successful; otherwise, an error.
///
/// ## Errors
/// * Returns an error if parameters are invalid (e.g., `parts` < `threshold`).
pub fn split(secret: &[u8], parts: usize, threshold: usize) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    // Validate the input parameters.
    if parts < threshold || parts > 255 || !(2..=255).contains(&threshold) || secret.is_empty() {
        return Err("invalid input parameters".into());
    }

    // Generate a sequence of non-zero values in GF(2^8)
    let mut x_coordinates: Vec<_> = (1..=255).collect();

    // Shuffle to create a random permutation of the x-coordinates.
    let mut rng = rand::thread_rng();
    rand::seq::SliceRandom::shuffle(x_coordinates.as_mut_slice(), &mut rng);

    // Set `share_size` to be equal to the length of the secret.
    let share_size = secret.len();
    // Initialize the output vector to store shares where each share
    // will consist of the y-coordinates plus one additional byte
    // for the x-coordinate.
    let mut shares = vec![vec![0u8; share_size + 1]; parts];

    // Assign the x-coordinates to the last position of each share.
    for idx in 0..parts {
        shares[idx][share_size] = x_coordinates[idx];
    }

    // For a polynomial of degree `k−1`, you need `k` distinct points to uniquely determine it,
    // therefor we generate a polynomial of degree `threshold - 1`.
    let degree = (threshold - 1) as u8;

    // For each byte in the secret, create a polynomial and evaluate it at each x-coordinate.
    for (s_idx, &secret_byte) in secret.iter().enumerate() {
        // Generate a polynomial for the current byte of the secret.
        let polynomial = Polynomial::generate(secret_byte, degree);

        for p_idx in 0..parts {
            // Access the x-coordinate for the current share.
            let x = x_coordinates[p_idx];
            // Evaluate the polynomial at the x-coordinate. This calculates
            // the y-value of the polynomial, effectively generating a part
            // of the share.
            let y = polynomial.evaluate(x);
            // Assign the evaluated y-value to the current share.
            shares[p_idx][s_idx] = y;
        }
    }

    Ok(shares)
}


/// Generates update keys and refreshes the shares
///
/// ## Arguments
/// * `shares` - Current shares to be refreshed
///
/// ## Returns
/// * Updated shares if successful; otherwise, an error.
///
/// ## Errors
/// * Returns an error if shares are inconsistent or invalid.
#[cfg(feature = "refresh")]
pub fn refresh(shares: &[Vec<u8>], threshold: usize) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    // Validate inputs
    let parts = shares.len();
    if parts < threshold || parts > 255 || !(2..=255).contains(&threshold) {
        return Err("invalid input parameters".into());
    }

    let share_size = shares[0].len();
    if !shares.iter().all(|share| share.len() == share_size) {
        return Err("inconsistent share lengths".into());
    }

    // The size of the data payload in each share, excluding the x-coordinate.
    let data_size = share_size - 1;

    // Create new shares with same dimensions
    let mut new_shares = shares.to_vec();

    // For a polynomial of degree `k−1`, you need `k` distinct points to uniquely determine it,
    // therefor we generate a polynomial of degree `threshold - 1`.
    let degree = (threshold - 1) as u8;

    for b_idx in 0..(data_size) {
        // Create random polynomial with f(0) = 0 to maintain the original secret
        let refresh_polynomial = Polynomial::generate(0, degree);

        // Update each share's byte at this position
        for (s_idx, share) in new_shares.iter_mut().enumerate() {
            // Get this share's x-coordinate (stored in last byte)
            let x_coordinate = shares[s_idx][data_size];
            // Calculate refresh value for this share at its x-coordinate
            let refresh_value = refresh_polynomial.evaluate(x_coordinate);
            // Add refresh value to share (GF(2^8) addition)
            share[b_idx] = ops::add(share[b_idx], refresh_value);
        }
    }

    Ok(new_shares)
}

/// Combines shares to reconstruct the secret.
///
/// ## Arguments
/// * `parts` - Shares of the secret.
///
/// ## Returns
/// * The original secret if successful; otherwise, an error.
///
/// ## Errors
/// * Returns an error if shares are inconsistent or insufficient.
pub fn combine(shares: &[Vec<u8>]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Validate the parts for consistency and sufficiency.
    if shares.len() < 2 || shares[0].len() < 2 {
        return Err("invalid parts".into());
    }

    // Ensure all parts are of the same length.
    let first_part_len = shares[0].len();
    for part in shares.iter().skip(1) {
        if part.len() != first_part_len {
            return Err("all parts must be the same length".into());
        }
    }

    // Initialize vectors to store the secret and the x and y samples.
    let mut secret = vec![0u8; first_part_len - 1];
    let mut x_samples = vec![0u8; shares.len()];
    let mut y_samples = vec![0u8; shares.len()];

    // Ensure that the x-coordinates are unique.
    let mut check_set = std::collections::HashSet::new();
    for (idx, part) in shares.iter().enumerate() {
        let sample = part[first_part_len - 1];
        if check_set.contains(&sample) {
            return Err("duplicate part detected".into());
        }
        check_set.insert(sample);
        x_samples[idx] = sample;
    }

    // Reconstruct each byte of the secret using polynomial interpolation.
    for idx in 0..(first_part_len - 1) {
        for (i, part) in shares.iter().enumerate() {
            y_samples[i] = part[idx];
        }
        let val = Polynomial::interpolate(&x_samples, &y_samples, 0);
        secret[idx] = val;
    }

    Ok(secret)
}

// Test cases for the `lib` module.
#[cfg(test)]
mod tests {
    use super::*;

    // The 'split' function with valid inputs.
    #[test]
    fn it_splits_secret() {
        let secret = b"test_secret";
        let threshold = 3;
        let parts = 5;

        // Split the secret into shares.
        let shares = split(secret, parts, threshold).expect("split failed");
        // The number of shares should match the specified number of parts.
        assert_eq!(shares.len(), parts);

        // Each share should be one byte longer than the secret to store the x-coordinate.
        for share in shares.iter() {
            assert_eq!(share.len(), secret.len() + 1);
        }
    }

    // The 'split' function with invalid inputs.
    #[test]
    fn it_fails_when_split_parts_less_than_thresshold() {
        let secret = b"test_secret";
        let threshold = 3;
        let parts = 2; // Less than the threshold

        assert!(split(secret, parts, threshold,).is_err());
    }

    // The 'combine' function with shares randomly generated from the split function.
    #[test]
    fn it_combines_from_random_shares() {
        let secret = [1, 2, 3]; // Original secret
        let threshold = 3;
        let parts = 5;

        let shares = split(&secret, parts, threshold).expect("split failed");
        // Choose a subset of shares that meet the threshold
        let selected_shares = &shares[..threshold];

        let reconstructed = combine(selected_shares).expect("combine failed");
        assert_eq!(reconstructed, secret);
    }

    // The `combine` function with known shares.
    #[test]
    fn it_combines_from_known_shares() {
        let secret = b"test_secret";
        // Valid known shares
        let shares = vec![
            vec![137, 206, 171, 244, 28, 176, 109, 4, 12, 168, 87, 50],
            vec![162, 176, 148, 45, 83, 38, 153, 204, 80, 141, 4, 1],
            vec![35, 165, 19, 114, 53, 31, 70, 25, 74, 248, 145, 132],
        ];

        // Combine the shares to reconstruct the secret.
        let reconstructed = combine(&shares).expect("combine failed");
        assert_eq!(reconstructed, secret);
    }

    // The 'combine' function with invalid or insufficient shares.
    #[test]
    fn it_fails_to_combine_invalid_shares_input() {
        // Inconsistent shares
        let shares = vec![vec![1, 2], vec![3, 4, 3]];
        assert!(combine(&shares).is_err());

        // Invalid number of shares
        let shares = vec![vec![1, 2]];
        assert!(combine(&shares).is_err());
    }

    // The 'combine' function with duplicate shares.
    #[test]
    fn it_fails_to_combine_duplicate_shares() {
        let shares = vec![
            vec![35, 165, 19, 114, 53, 31, 70, 25, 74, 248, 145, 132],
            //duplicate shares
            vec![137, 206, 171, 244, 28, 176, 109, 4, 12, 168, 87, 50],
            vec![137, 206, 171, 244, 28, 176, 109, 4, 12, 168, 87, 50],
        ];

        assert!(combine(&shares).is_err());
    }
    // Test basic refresh functionality
    #[test]
    #[cfg(feature = "refresh")]

    fn it_refreshes_shares() {
        let secret = b"test_secret";
        let threshold = 3;
        let shares = split(secret, 5, threshold).expect("split failed");

        let refreshed_shares = refresh(&shares, threshold).expect("refresh failed");

        // Verify refreshed shares can reconstruct secret
        let selected_shares = &refreshed_shares[..threshold];
        let reconstructed = combine(selected_shares).expect("combine failed");
        assert_eq!(reconstructed, secret);
    }

    // Test refresh with known shares
    #[test]
    #[cfg(feature = "refresh")]

    fn it_refreshes_known_shares() {
        // Valid known shares
        let shares = vec![
            vec![137, 206, 171, 244, 28, 176, 109, 4, 12, 168, 87, 50],
            vec![162, 176, 148, 45, 83, 38, 153, 204, 80, 141, 4, 1],
            vec![35, 165, 19, 114, 53, 31, 70, 25, 74, 248, 145, 132],
        ];
        let threshold = 3;

        let refreshed_shares = refresh(&shares, threshold).expect("refresh failed");
        let reconstructed = combine(&refreshed_shares[..threshold]).expect("combine failed");
        assert_eq!(reconstructed, b"test_secret");
    }

    // Test refresh with invalid inputs
    #[test]
    #[cfg(feature = "refresh")]

    fn it_fails_to_refresh_invalid_shares() {
        // Inconsistent shares
        let shares = vec![vec![1, 2], vec![3, 4, 3]];
        assert!(refresh(&shares, 2).is_err());

        // Invalid threshold - threshold must be at least 2
        let shares = vec![vec![1, 2], vec![3, 4]];
        assert!(refresh(&shares, 1).is_err());

        // Threshold larger than number of shares
        let shares = vec![vec![1, 2], vec![3, 4]];
        assert!(refresh(&shares, 3).is_err());
    }
}
