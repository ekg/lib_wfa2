# Understanding MemoryMode::Ultralow in lib_wfa2

## Overview

`MemoryMode::Ultralow` is a critical setting for enabling the bidirectional wavefront alignment (bi-WFA) algorithm in WFA2-lib. However, there are important considerations about how and when to set this mode in the Rust wrapper.

## What is MemoryMode::Ultralow?

`MemoryMode::Ultralow` enables the most memory-efficient variant of the wavefront alignment algorithm:
- Uses bidirectional wavefront alignment (bi-WFA)
- Aligns from both ends simultaneously and meets in the middle
- Drastically reduces memory usage for long sequences
- Essential for aligning sequences longer than a few kilobases

## Current Implementation Issue

There's a potential issue in how memory mode is handled when creating aligners with `with_penalties_affine2p`:

```rust
// In with_penalties_affine2p()
unsafe {
    let mut attributes = wfa::wavefront_aligner_attr_default;
    attributes.distance_metric = wfa::distance_metric_t_gap_affine_2p;
    // ...set penalties...
    
    // ISSUE: Memory mode is hardcoded to High!
    attributes.memory_mode = wfa::wavefront_memory_t_wavefront_memory_high;
    
    let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);
}
```

This means that even if you later call `set_memory_mode(MemoryMode::Ultralow)`, the aligner may have already been initialized with high memory mode settings.

## How WFMASH Does It Correctly

Looking at the C++ code in WFMASH, they pass the memory mode directly to the constructor:

```cpp
wfa::WFAlignerGapAffine2Pieces wf_aligner(
    0,                              // match
    penalties.mismatch,             // mismatch
    penalties.gap_opening1,         // gap opening 1
    penalties.gap_extension1,       // gap extension 1
    penalties.gap_opening2,         // gap opening 2
    penalties.gap_extension2,       // gap extension 2
    wfa::WFAligner::Alignment,      // alignment scope
    wfa::WFAligner::MemoryUltralow // memory mode - passed at creation!
);
```

## Recommended Usage Patterns

### Option 1: Create Custom Constructor (Ideal)

The best solution would be to modify the Rust wrapper to accept memory mode at creation:

```rust
// This would be the ideal API (not currently implemented)
let aligner = AffineWavefronts::with_penalties_affine2p_and_mode(
    0, 4, 6, 2, 12, 1,              // penalties
    MemoryMode::Ultralow            // memory mode
);
```

### Option 2: Use Default Constructor (Current Workaround)

Since the default constructor uses NULL attributes (which lets WFA2 use its defaults), you might have better luck with:

```rust
// Create with default constructor
let mut aligner = AffineWavefronts::default();

// Set penalties after creation
aligner.set_penalties_affine2p(0, 4, 6, 2, 12, 1);

// Set memory mode
aligner.set_memory_mode(MemoryMode::Ultralow);

// Set other options
aligner.set_alignment_scope(AlignmentScope::Alignment);
aligner.set_heuristic(&HeuristicStrategy::None);
```

### Option 3: Verify Memory Mode is Working

You can verify if ultralow memory mode is actually being used:

```rust
// After setting memory mode
aligner.set_memory_mode(MemoryMode::Ultralow);

// Verify it was set
match aligner.get_memory_mode() {
    MemoryMode::Ultralow => println!("✓ Ultralow memory mode active"),
    mode => println!("✗ Wrong memory mode: {:?}", mode),
}
```

## Testing Memory Mode Effectiveness

To verify bi-WFA is working correctly:

1. **Memory Usage Test**: Align very long sequences (>100kb) and monitor memory usage
2. **Performance Pattern**: Bi-WFA should have different performance characteristics
3. **Debug Output**: Check if WFA2 internal logs mention bidirectional alignment

```rust
// Test with long sequences
let long_query = vec![b'A'; 100_000];
let long_ref = vec![b'A'; 100_000];

// Should use minimal memory with Ultralow mode
let status = aligner.align(&long_query, &long_ref);
```

## Potential Fix for the Wrapper

The `with_penalties_affine2p` function should be modified to:

```rust
pub fn with_penalties_affine2p(
    match_: i32,
    mismatch: i32,
    gap_opening1: i32,
    gap_extension1: i32,
    gap_opening2: i32,
    gap_extension2: i32,
) -> Self {
    unsafe {
        let mut attributes = wfa::wavefront_aligner_attr_default;
        attributes.distance_metric = wfa::distance_metric_t_gap_affine_2p;
        
        // Set penalties
        attributes.affine2p_penalties.match_ = match_;
        attributes.affine2p_penalties.mismatch = mismatch;
        attributes.affine2p_penalties.gap_opening1 = gap_opening1;
        attributes.affine2p_penalties.gap_extension1 = gap_extension1;
        attributes.affine2p_penalties.gap_opening2 = gap_opening2;
        attributes.affine2p_penalties.gap_extension2 = gap_extension2;
        
        // SHOULD USE: wavefront_memory_ultralow instead of high!
        attributes.memory_mode = wfa::wavefront_memory_t_wavefront_memory_ultralow;
        
        // Disable heuristic
        attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_none;
        
        let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);
        Self { wf_aligner }
    }
}
```

## Summary

1. **Current Issue**: `with_penalties_affine2p` hardcodes memory mode to High
2. **Impact**: `set_memory_mode(MemoryMode::Ultralow)` after creation may not fully enable bi-WFA
3. **Workaround**: Use default constructor and set all parameters afterward
4. **Verification**: Always verify the memory mode is correctly set
5. **Long-term Fix**: The wrapper should allow memory mode specification at creation time

For reliable bi-WFA alignment with ultralow memory usage, use the default constructor pattern shown in Option 2 until the wrapper is updated to properly support memory mode configuration at initialization.