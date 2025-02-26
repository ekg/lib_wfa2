# libwfa2

Rust binding for [WFA2-lib](https://github.com/smarco/WFA2-lib), with support for both affine gap and dual-cost gap-affine penalties.

## Building

To use `libwfa2` in `your project`, you need to build `WFA2-lib` first:

```shell
git clone https://github.com/smarco/WFA2-lib
cd WFA2-lib
make clean all
```

and then:

```bash
# Point to your pre-built WFA2-lib directory
export WFA2LIB_PATH="/path/to/WFA2-lib"

# Build your project
git clone <your-project>
cd <your-project>
cargo build --release
```

## Examples

With affine gap penalties:

```rust
use libwfa2::affine_wavefront::AffineWavefronts;

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
use libwfa2::affine_wavefront::AffineWavefronts;

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
use libwfa2::affine_wavefront::{AffineWavefronts, HeuristicStrategy};

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
