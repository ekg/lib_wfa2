# Memory Mode Implementation Summary

## The Issue with `set_memory_mode()`

Our testing revealed that `set_memory_mode()`:
1. **Changed the memory_mode field** in the struct
2. **Did NOT reconfigure the WFA2 aligner** 

This meant:
- If you created an aligner with Ultralow mode and then called `set_memory_mode(High)`:
  - `get_memory_mode()` would return `High` 
  - But the aligner would still use bi-WFA (ultralow) internally
  - Scores would still be INT_MIN
  - Memory usage would still be ultralow

## Why This Happened

WFA2-lib initializes different internal structures based on memory mode at creation time:
- **High/Medium/Low**: Standard wavefront structures
- **Ultralow**: Bidirectional aligner (`bialigner`) structure

Simply changing the field didn't reinitialize these structures.

## The Solution

We've now:
1. **REMOVED `set_memory_mode()`** completely to prevent confusion
2. **Added proper constructors** that set memory mode at creation time
3. **Provided a builder pattern** for complex configurations

## Correct Usage

### ❌ Wrong Way
```rust
let mut aligner = AffineWavefronts::default();
aligner.set_memory_mode(MemoryMode::Ultralow); // Won't actually use ultralow!
```

### ✅ Right Way
```rust
// Option 1: Direct constructor
let aligner = AffineWavefronts::new_ultralow();

// Option 2: With explicit memory mode
let aligner = AffineWavefronts::with_penalties_affine2p_and_memory_mode(
    0, 4, 6, 2, 12, 1,
    MemoryMode::Ultralow
);

// Option 3: Builder pattern
let aligner = AffineWavefrontsBuilder::new()
    .penalties(0, 4, 6, 2)
    .dual_affine_penalties(12, 1)
    .memory_mode(MemoryMode::Ultralow)
    .build();
```

## Test Results Confirm

Our tests show that when properly initialized with Ultralow:
- ✅ Valid CIGAR strings are produced
- ✅ Memory usage is minimal
- ⚠️ Scores show as INT_MIN (characteristic of bi-WFA)
- ✅ Works with both gap-affine and gap-affine-2p

The key takeaway: **Memory mode must be set at creation time, not after!**