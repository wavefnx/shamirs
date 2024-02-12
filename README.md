<h1 align="center"> ShamiRS </h1>

<div align="center"> 
    
[Overview](#Overview) | [Disclaimer](#Disclaimer) | [Security](#Security) | [Acknowledgments](#Acknowledgments) | [Tests](#Tests) | [Installation](#Installation) | [Usage](#Usage) | [Examples](#Examples) | [License](#License)
</div>

<div align="center">
    
[![CI](https://img.shields.io/github/actions/workflow/status/wavefnx/shamirs/ci.yml?style=flat-square&label=CI&labelColor=%23343940&color=%2340C057)](https://github.com/wavefnx/shamirs/actions/workflows/ci.yml)
[![MPL-2.0](https://img.shields.io/github/license/wavefnx/shamirs?style=flat-square&color=blue&label=)](LICENSE)
</div>


<div align="center">
    
![shamirs-logo](https://github.com/wavefnx/shamirs/assets/157986149/c3cb34a4-646f-431e-81da-73327ad98c8d)
</div>

## Overview
ShamiRS (shamir-rs) utilizes [Sharmir's Secret Sharing](https://en.wikipedia.org/wiki/Shamir%27s_secret_sharing), a cryptographic method for dividing a secret into multiple shares. In this context, the secret is represented as a polynomial in Galois Field `GF(2^8)` where each share corresponds to a point on this polynomial.

- **Share Generation**: Each byte of the secret is assigned as the constant term of a distinct, newly created polynomial, while it's remaining coefficients are being randomly generated. Shares are then produced by evaluating these polynomials at random `x` points with each share `s=(y1, y2, .., yn, x)` consisting of an `x` point and its corresponding `y1..yn` points being the result of every evaluation.
- **Secret Reconstruction**: When the minimum number of shares (`threshold`)  are gathered, the polynomial can be then reconstructed using Lagrange interpolation. The constant terms of these resulting polynomials would be each byte of the original secret.

A key aspect of this method is that possession of less shares than the required `threshold` amount reveals no information about the secret, which substantially reduces the risk of a single point of failure.

## Disclaimer
The library has not been subjected to a formal security audit. It should be used at your own discretion and risk. Furthermore, backward compatibility is not guaranteed and the package is intentionally not published on crates.io until and if there's a `stable` release in the future.

Contributions are welcome. Users are encouraged to submit pull requests, fork, or alter the code in accordance with the terms outlined in the [LICENSE](#LICENSE).

## Security
ShamiRS places strong emphasis on security.

- **CSPRNG**: To maximize unpredictability in scenarios requiring randomness, the library utilizes only generators from the `rand` crate that are classified as cryptograpically secure.
- **Constant-Time Operations**: To mitigate side-channel (timing) attacks, constant-time operations are implemented where necessary.
- **No LOG/EXP Tables**: Arithmetic operations in `GF(2^8)` are handled through bitwise and constant-time operations, avoiding the security risks associated with precomputed `LOG/EXP` tables. ([CVE-2023-25000](https://github.com/advisories/GHSA-vq4h-9ghm-qmrr)).

The original library mentioned in the CVE report `2023-25000` has already mitigated the vulnerability, although it's crucial to exercise caution when using other libraries to verify that they do not utilize precomputed `LOG/EXP` tables, which in this case are susceptible to security vulnerabilities.

## Acknowledgments
The current implementation is based on the Golang library developed by Hashicorp. You can find the repository [here](https://github.com/hashicorp/vault/blob/main/shamir/shamir.go).

## Tests
Every module has it's own tests located at the end of the file, to execute them run the following command at the root of the repository:  

```rust
cargo test
```

## Installation
```toml
[dependencies]
shamirs = { git = "https://github.com/wavefnx/shamirs" }
```

## Usage
```rust
use shamirs::{combine, split};

// The total number of shares the secret will be split into.
const PARTS: usize = 5;
// The minimum number of shares required to reconstruct the secret.
const THRESHOLD: usize = 3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set the bytes of the secret.
    let secret: &[u8] = b"example_secret";
    // 65 78 61 6d 70 6c 65 5f 73 65 63 72 65 74

    // Split the secret into shares.
    let shares: Vec<Vec<u8>> = split(secret, PARTS, THRESHOLD)?;
    // This will generate `PARTS` amount of shares, with their corresponding coordinates.
    //      y0   y1   y2   y3   y4   y5   y6   y7   y8   y9  y10  y11  y12  y13   x
    // -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
    // s1 = 67 | 7c | cf | fd | 45 | 77 | 67 | fe | db | a2 | d9 | 62 | 0b | c2 | 82
    // s2 = 46 | 50 | 1f | 7b | 38 | cc | 46 | eb | 90 | dc | a1 | 5e | 37 | 48 | 45
    // ...
    
    // Combine a subset of `THRESHOLD` amount of shares to recover the secret.
    let reconstructed = combine(&shares[..THRESHOLD])?;

    // Verify that the recovered secret matches the original secret.
    assert_eq!(secret, reconstructed);

    Ok(())
}
```

## Examples
To execute the [basic example](examples/basic.rs), run the following command at the root of the repository: 
```rust
cargo run --example basic -q

// cargo run: Invokes the Rust package manager to run the code.
// --example basic: Specifies that you want to run the 'basic' example.
// -q: Optional flag for quiet mode, which reduces unnecessary output.
```

the output should be similar to:

```sh
share_1: 3b89c1886bf64f60176b2c429d730d3ca13d7f95b0ac380686fe948c5acae113d0
share_2: 220bd4708ef623b926f361ab0a3be001ed29c4e73ae246cfa84e321f2646430f4f
share_3: 15c45226d2c8582909cb8472effc35e1188e0321c7d5d7b56b66652839c8bc7555
share_4: 9fb2d8cf52addd0c5f7a8c35e8b807f94cceeeaa61582ecc062d59da073728f388
share_5: 099c6e7726618fe38fcdd64173067f18320a8d1d58638b7f83a2471e7bf1472570

initial:   dde23b13ab8bcddaf1ee97bb6206408a1dc9a7bd656e0b96a795ea7cbf27abeb
recovered: dde23b13ab8bcddaf1ee97bb6206408a1dc9a7bd656e0b96a795ea7cbf27abeb
the secret was successfully reconstructed.
```

## License
This library is released under the terms of the [Mozilla Public License](https://www.mozilla.org/en-US/MPL/) version 2.0. See [LICENSE](LICENSE).
