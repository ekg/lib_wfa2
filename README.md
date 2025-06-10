# lib_wfa2

Rust binding for [WFA2-lib](https://github.com/smarco/WFA2-lib), with support for both affine gap and dual-cost gap-affine penalties.

## Usage

To use `lib_wfa2`, add the following to your `Cargo.toml`:

```toml
[dependencies]
lib_wfa2 = { git = "https://github.com/AndreaGuarracino/lib_wfa2" }
```

Note that this library requires C build tools (`gcc`, `make`) to compile the underlying `WFA2-lib`.

## Building

To build `lib_wfa2`, simply clone the repository with submodules and build it:

```bash
git clone --recursive https://github.com/AndreaGuarracino/lib_wfa2
cd lib_wfa2
cargo build --release
```

The build process automatically compiles the included `WFA2-lib`.

## Examples

With affine gap penalties:

```rust
use lib_wfa2::affine_wavefront::AffineWavefronts;

pub fn main() {
    let aligner = AffineWavefronts::default();

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

With dual-cost gap-affine penalties:

```rust
use lib_wfa2::affine_wavefront::AffineWavefronts;

pub fn main() {
    // Create an aligner with affine2p penalties.
    let aligner = AffineWavefronts::with_penalties_affine2p(0, 6, 4, 2, 12, 1);

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

Setting heuristics:

```rust
use lib_wfa2::affine_wavefront::{AffineWavefronts, HeuristicStrategy};

pub fn main() {
    let mut aligner = AffineWavefronts::default();

    aligner.set_heuristic(&HeuristicStrategy::BandedStatic { band_min_k: -1, band_max_k: 1 });

    // pattern means query and text means reference
    let pattern = b"TCTTTACTCGCGCGTTGGAGAAATACAATAGT";
    let text = b"TCTATACTGCGCGTTTGGAGAAATAAAATAGT";

    let _status = aligner.align(pattern, text);

    println!("Pattern: {}", String::from_utf8_lossy(pattern));
    println!("Text:    {}\n", String::from_utf8_lossy(text));

    println!("Score: {}", aligner.score());
    println!("Cigar: {}", String::from_utf8_lossy(aligner.cigar()));
}
```
