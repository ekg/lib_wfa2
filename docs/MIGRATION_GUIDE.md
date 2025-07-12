# Migration Guide: set_memory_mode() Removal

## Why Was `set_memory_mode()` Removed?

The `set_memory_mode()` method was removed because it was fundamentally broken:
- It only changed a field value but didn't reconfigure the WFA2 aligner
- This led to confusing behavior where the reported memory mode didn't match actual behavior
- Memory mode MUST be set at aligner creation time in WFA2-lib

## Migration Examples

### Old Code (Broken)
```rust
// This appeared to work but didn't actually use ultralow memory!
let mut aligner = AffineWavefronts::default();
aligner.set_memory_mode(MemoryMode::Ultralow);
aligner.set_penalties_affine2p(0, 4, 6, 2, 12, 1);
```

### New Code (Correct)

#### Option 1: Use the new constructors
```rust
// For gap-affine with custom memory mode
let aligner = AffineWavefronts::with_penalties_and_memory_mode(
    0, 4, 6, 2,
    MemoryMode::Ultralow
);

// For gap-affine-2p with custom memory mode
let aligner = AffineWavefronts::with_penalties_affine2p_and_memory_mode(
    0, 4, 6, 2, 12, 1,
    MemoryMode::Ultralow
);
```

#### Option 2: Use the convenience constructor
```rust
// Quick setup for bi-WFA with ultralow memory
let aligner = AffineWavefronts::new_ultralow();
```

#### Option 3: Use the builder pattern
```rust
let aligner = AffineWavefrontsBuilder::new()
    .penalties(0, 4, 6, 2)
    .dual_affine_penalties(12, 1)
    .memory_mode(MemoryMode::Ultralow)
    .alignment_scope(AlignmentScope::Alignment)
    .heuristic(HeuristicStrategy::None)
    .build();
```

## Key Points

1. **Memory mode is immutable** after aligner creation
2. **Choose the right constructor** based on your needs
3. **Ultralow mode** will return INT_MIN scores but valid CIGARs
4. **All memory modes** now work correctly when set at creation

## Compiler Error?

If you get:
```
error[E0599]: no method named `set_memory_mode` found
```

This is expected! Replace your code with one of the patterns above.