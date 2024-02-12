use rand::{prelude::SliceRandom, RngCore};
use shamirs::{combine, split};

/// The total number of parts the secret will be split into.
const PARTS: usize = 5;
/// The minimum number of parts needed to reconstruct the secret.
const THRESHOLD: usize = 3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Init secret buffer.
    let mut secret: [u8; 32] = [0; 32];
    // Fill the buffer with random bytes.
    rand::thread_rng().fill_bytes(&mut secret);

    // Split the secret into shares.
    let mut shares: Vec<Vec<u8>> = split(&secret, PARTS, THRESHOLD)?;
    // This will generate `PARTS` amount of shares, with their corresponding coordinates.
    //      y0   y1   y2   y3   y4   y5   y6   y7   y8   y9  y10  y11  y12  y13   x
    // -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
    // s1 = 67 | 7c | cf | fd | 45 | 77 | 67 | fe | db | a2 | d9 | 62 | 0b | c2 | 82
    // s2 = 46 | 50 | 1f | 7b | 38 | cc | 46 | eb | 90 | dc | a1 | 5e | 37 | 48 | 45
    // ...

    // For demonstration purposes, we shuffle the shares in order to pick `THRESHOLD` amount of random ones later.
    shares.shuffle(&mut rand::thread_rng());

    // Print all shares in hex encoding.
    // In non-demonstrative scenarios, treat each share as a secret and handle them securely,
    // even if individually they don't contain any sensitive information.
    for (i, share) in shares.iter().enumerate() {
        println!("share_{}: {}", i + 1, hex::encode(share));
    }

    // Combine a subset of the shares to recover the secret.
    // Here, the first `THRESHOLD` amount of the shuffled shares are used for recovery,
    // although any `THRESHOLD` amount of shares can be used.
    let selected_shares: Vec<Vec<u8>> = shares[..THRESHOLD].to_vec();
    let recovered_secret: Vec<u8> = combine(&selected_shares)?;

    // Verify that the recovered secret matches the original secret.
    assert_eq!(secret, recovered_secret.as_slice());

    // Finally, print the original and the recovered secret in hex.
    println!();
    println!("initial:   {}", hex::encode(&secret));
    println!("recovered: {}", hex::encode(&recovered_secret));

    println!("the secret was successfully reconstructed.");

    Ok(())
}
