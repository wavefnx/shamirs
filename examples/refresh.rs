#[cfg(feature = "refresh")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rand::{thread_rng, RngCore};
    use shamirs::{combine, refresh, split};

    /// The total number of shares the secret will be split into.
    const NUM_SHARES: usize = 5;
    /// The minimum number of shares needed to reconstruct the secret.
    const THRESHOLD: usize = 3;

    // ----------------------------------------
    // --- Stage I: Initial Share Generation --
    // ----------------------------------------

    // Initialize secret buffer with random bytes
    let mut secret = [0u8; 32];
    thread_rng().fill_bytes(&mut secret);
    println!("secret: {}\n", hex::encode(&secret));

    // Split the secret into shares
    println!("# initial shares:");
    let shares = split(&secret, NUM_SHARES, THRESHOLD)?;
    // This will generate `NUM_SHARES` amount of shares, with their corresponding x-coordinates.
    //      y0   y1   y2   y3   y4   y5   y6   y7   y8   y9  y10  y11  y12  y13   x
    // -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
    // s1 = 67 | 7c | cf | fd | 45 | 77 | 67 | fe | db | a2 | d9 | 62 | 0b | c2 | 82
    // s2 = 46 | 50 | 1f | 7b | 38 | cc | 46 | eb | 90 | dc | a1 | 5e | 37 | 48 | 45
    // ...

    // Print all shares in hex encoding
    for (i, share) in shares.iter().enumerate() {
        println!("share_{}: {}", i + 1, hex::encode(share));
    }

    // Combine subset of shares to recover secret
    let recovered = combine(&shares[..THRESHOLD])?;
    assert_eq!(secret, recovered.as_slice());

    println!("\nrecovered: {}", hex::encode(&recovered));
    println!("secret successfully reconstructed from shares.\n");

    // ----------------------------------------
    // ---   Stage II: Proactive Refresh    ---
    // ----------------------------------------

    // Refresh the shares
    println!("# refreshed shares:");
    let refreshed = refresh(&shares, THRESHOLD)?;

    // Print all refreshed shares in hex encoding
    for (i, share) in refreshed.iter().enumerate() {
        println!("share_{}: {}", i + 1, hex::encode(share));
    }

    // Combine a subset of the refreshed shares to recover the secret
    let final_secret = combine(&refreshed[..THRESHOLD])?;
    assert_eq!(secret, final_secret.as_slice());

    println!("\nrecovered: {}", hex::encode(&final_secret));
    println!("secret successfully reconstructed from refreshed shares.");

    Ok(())
}

// Fallback function if the refresh feature is not enabled
#[cfg(not(feature = "refresh"))]
fn main() {
    println!("refresh feature is not enabled");
    println!("use: cargo run --example refresh --features refresh -q");
}
