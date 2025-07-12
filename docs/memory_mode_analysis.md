# Analysis: MemoryMode::Ultralow in lib_wfa2

## The Problem

You're right to be concerned. After analyzing the code, there are **two critical issues** that prevent proper use of MemoryMode::Ultralow in the current Rust wrapper:

### Issue 1: Hardcoded Memory Mode in Constructor

In `with_penalties_affine2p()`, the memory mode is hardcoded to `High`:

```rust
// src/affine_wavefront.rs:249
attributes.memory_mode = wfa::wavefront_memory_t_wavefront_memory_high;
```

This means any aligner created with `with_penalties_affine2p()` starts with High memory mode.

### Issue 2: Post-Creation Memory Mode Changes May Not Work

The more serious issue is that changing memory mode after the aligner is created might not have the intended effect. Here's why:

1. **WFA2 Initialization**: When `wavefront_aligner_new()` is called, WFA2 likely initializes internal data structures based on the memory mode
2. **Bidirectional Aligner**: The bi-WFA (ultralow) mode requires a special `bialigner` component to be initialized
3. **Late Changes**: Simply changing the `memory_mode` field after creation doesn't reinitialize these structures

Evidence from the C structure:
```c
// The aligner has a bialigner pointer that's only used in ultralow mode
pub struct _wavefront_aligner_t {
    // ...
    pub memory_mode: wavefront_memory_t,
    // ...
    pub bialigner: *mut wavefront_bialigner_t,  // This needs special initialization!
}
```

## Verification

The Rust wrapper's `set_memory_mode()` function only updates the field value:

```rust
pub fn set_memory_mode(&mut self, mode: MemoryMode) {
    (unsafe { *self.wf_aligner }).memory_mode = match mode {
        MemoryMode::Ultralow => wfa::wavefront_memory_t_wavefront_memory_ultralow,
        // ...
    }
}
```

This is just changing a number in a struct - it doesn't call any C functions to reconfigure the aligner!

## Why This Matters

Without proper ultralow mode:
1. You're not getting bi-directional wavefront alignment
2. Memory usage will be much higher than expected
3. Large sequences may fail or use excessive memory
4. You're essentially getting standard WFA instead of bi-WFA

## The Solution

### Immediate Workaround

Unfortunately, there's **no reliable workaround** with the current wrapper. The memory mode must be set at creation time in the C library.

### Proper Fix Required

The Rust wrapper needs to be modified to:

1. **Option A**: Add a new constructor that accepts memory mode:
```rust
pub fn with_penalties_affine2p_mode(
    match_: i32,
    mismatch: i32,
    gap_opening1: i32,
    gap_extension1: i32,
    gap_opening2: i32,
    gap_extension2: i32,
    memory_mode: MemoryMode,  // Add this parameter
) -> Self {
    unsafe {
        let mut attributes = wfa::wavefront_aligner_attr_default;
        attributes.distance_metric = wfa::distance_metric_t_gap_affine_2p;
        // ... set penalties ...
        
        // Use the provided memory mode
        attributes.memory_mode = match memory_mode {
            MemoryMode::Ultralow => wfa::wavefront_memory_t_wavefront_memory_ultralow,
            // ... other modes ...
        };
        
        let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);
        Self { wf_aligner }
    }
}
```

2. **Option B**: Make `set_memory_mode()` recreate the aligner:
```rust
pub fn set_memory_mode(&mut self, mode: MemoryMode) {
    // This would need to:
    // 1. Save current settings
    // 2. Delete current aligner
    // 3. Create new aligner with new memory mode
    // This is complex and error-prone
}
```

## Confirming the Issue

You can verify this is a problem by:

1. Running the test program I created (`examples/test_memory_mode.rs`)
2. Checking if large sequence alignments use expected memory
3. Looking for bi-aligner initialization in debug logs

## Recommendation

**The current lib_wfa2 wrapper cannot properly use MemoryMode::Ultralow** when using `with_penalties_affine2p()`. This is a significant limitation that needs to be fixed in the wrapper code.

For now, you'll need to either:
1. Fork and fix the wrapper
2. Use the C++ API directly
3. Accept the memory limitations
4. Use shorter sequences that work with high memory mode

This explains why you're seeing issues - the wrapper simply doesn't support this critical feature properly.