# libwfa2

Rust bindings for the wavefront algorithm (WFA2-lib,[ https://github.com/smarco/WFA2-lib](https://github.com/smarco/WFA2-lib "Link to repository")). This library supports both the standard affine gap penalties and the newer affine2p penalty variant. This is inspired by the rust crate libwfa ([crate](https://crates.io/crates/libwfa "cargo crate"), [github]()).

## Example

Basic usage of the library.

```
use libwfa2::affine_wavefront::AffineWavefronts;

pub fn main() {
    let aligner = AffineWavefronts::default();

    // pattern means query
    let pattern = b"TCTTTACTCGCGCGTTGGAGAAATACAATAGT";

    // Text means reference
    let text = b"TCTATACTGCGCGTTTGGAGAAATAAAATAGT";

    aligner.align(pattern, text);

    println!("Pattern: {}", String::from_utf8_lossy(pattern));
    println!("Text:    {}\n", String::from_utf8_lossy(text));

    println!("Score: {}", aligner.score());
    println!("Cigar: {}", String::from_utf8_lossy(aligner.cigar()));
}
```

### Using Affine2p Penalties

You can also create an aligner using the affine 2p penalties. This variant allows specifying a second gap opening and gap extension penalty. For example:

```rust
use libwfa2::affine_wavefront::AffineWavefronts;

pub fn main() {
    // Create an aligner with affine2p penalties.
    // Here: match = 1, mismatch = -1, gap_opening1 = -3, gap_extension1 = -1,
    // gap_opening2 = -5, gap_extension2 = -1.
    let aligner = AffineWavefronts::with_penalties_affine2p(1, -1, -3, -1, -5, -1);

    // pattern means query and text means reference
    let pattern = b"TCTTTACTCGCGCGTTGGAGAAATACAATAGT";
    let text = b"TCTATACTGCGCGTTTGGAGAAATAAAATAGT";

    aligner.align(pattern, text);

    println!("Pattern: {}", String::from_utf8_lossy(pattern));
    println!("Text:    {}\n", String::from_utf8_lossy(text));

    println!("Score: {}", aligner.score());
    println!("Cigar: {}", String::from_utf8_lossy(aligner.cigar()));
}
```

This example creates an affine wavefront aligner using the affine2p penalty configuration.
```

Setting heuristics

```
use libwfa2::affine_wavefront::{AffineWavefronts, HeuristicStrategy};

pub fn main() {
    println!("Example2\n");

    let mut aligner = AffineWavefronts::default();

    aligner.set_heuristic(&HeuristicStrategy::BandedStatic { band_min_k: -1, band_max_k: 1 });

    // pattern means query
    let pattern = b"TCTTTACTCGCGCGTTGGAGAAATACAATAGT";

    // Text means reference
    let text = b"TCTATACTGCGCGTTTGGAGAAATAAAATAGT";

    let _status = aligner.align(pattern, text);

    println!("Pattern: {}", String::from_utf8_lossy(pattern));
    println!("Text:    {}\n", String::from_utf8_lossy(text));

    println!("Score: {}", aligner.score());
    println!("Cigar: {}", String::from_utf8_lossy(aligner.cigar()));
}
```
