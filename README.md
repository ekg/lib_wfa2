# lib_wfa2

[![CI](https://github.com/your-username/lib_wfa2/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/lib_wfa2/actions/workflows/ci.yml)

Rust binding for [WFA2-lib](https://github.com/smarco/WFA2-lib), with support for both affine gap and dual-cost gap-affine penalties.

📚 **[Full documentation available in the docs/ directory](docs/)**

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

### Basic Usage with Affine Gap Penalties

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

### Ultra-low Memory Mode (bi-WFA)

For aligning long sequences with minimal memory usage:

```rust
use lib_wfa2::affine_wavefront::{AffineWavefronts, MemoryMode};

pub fn main() {
    // Quick constructor for ultralow memory with dual-cost gap-affine
    let aligner = AffineWavefronts::new_ultralow();

    // Or specify memory mode explicitly
    let aligner = AffineWavefronts::with_penalties_affine2p_and_memory_mode(
        0, 4, 6, 2, 12, 1, 
        MemoryMode::Ultralow
    );

    let pattern = b"TCTTTACTCGCGCGTTGGAGAAATACAATAGT";
    let text = b"TCTATACTGCGCGTTTGGAGAAATAAAATAGT";

    aligner.align(pattern, text);

    println!("Score: {}", aligner.score());
    println!("Memory mode: {:?}", aligner.get_memory_mode());
}
```

### Builder Pattern for Complex Configurations

```rust
use lib_wfa2::affine_wavefront::{AffineWavefrontsBuilder, MemoryMode, AlignmentScope, HeuristicStrategy};

pub fn main() {
    let aligner = AffineWavefrontsBuilder::new()
        .penalties(0, 4, 6, 2)           // match, mismatch, gap_open, gap_ext
        .dual_affine_penalties(12, 1)    // gap_open2, gap_ext2
        .memory_mode(MemoryMode::Ultralow)
        .alignment_scope(AlignmentScope::Alignment)
        .heuristic(HeuristicStrategy::None)
        .build();

    let pattern = b"TCTTTACTCGCGCGTTGGAGAAATACAATAGT";
    let text = b"TCTATACTGCGCGTTTGGAGAAATAAAATAGT";

    let _status = aligner.align(pattern, text);

    println!("Score: {}", aligner.score());
    println!("Cigar: {}", String::from_utf8_lossy(aligner.cigar()));
}
```

### Setting Heuristics

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
