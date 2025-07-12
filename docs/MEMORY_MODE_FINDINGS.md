# Memory Mode Implementation Findings

## Summary

After implementing proper memory mode support in lib_wfa2, here are the key findings:

### ✅ What Works

1. **All memory modes are now configurable at creation time**:
   - `MemoryMode::High` - Standard WFA with full DP matrix
   - `MemoryMode::Medium` - Reduced memory with pruning
   - `MemoryMode::Low` - Aggressive pruning
   - `MemoryMode::Ultralow` - Bidirectional WFA (bi-WFA)

2. **All combinations are supported**:
   - Gap-affine (1-piece) + all memory modes ✓
   - Gap-affine-2p (dual-cost) + all memory modes ✓
   - All produce valid CIGAR strings ✓

3. **New API methods**:
   ```rust
   // Quick ultralow constructor
   let aligner = AffineWavefronts::new_ultralow();
   
   // Explicit memory mode
   let aligner = AffineWavefronts::with_penalties_affine2p_and_memory_mode(
       0, 4, 6, 2, 12, 1,
       MemoryMode::Ultralow
   );
   
   // Builder pattern
   let aligner = AffineWavefrontsBuilder::new()
       .penalties(0, 4, 6, 2)
       .dual_affine_penalties(12, 1)
       .memory_mode(MemoryMode::Ultralow)
       .build();
   ```

### ⚠️ Known Issues

1. **Score retrieval with Ultralow mode**:
   - Returns INT_MIN (-2147483648) instead of actual score
   - CIGAR is still valid and correct
   - This appears to be a characteristic of bi-WFA in WFA2-lib
   - May need special handling to retrieve scores from bi-aligner

2. **set_memory_mode() limitation**:
   - Only changes the field value, doesn't reconfigure the aligner
   - Memory mode MUST be set at creation time
   - This is a fundamental limitation of how WFA2-lib initializes

3. **Distance metric changes**:
   - `set_penalties_affine2p()` doesn't change the distance metric
   - Distance metric must also be set at creation time

## Test Results

| Configuration | High | Medium | Low | Ultralow |
|--------------|------|--------|-----|----------|
| Gap-affine | ✓ Score: -24 | ✓ Score: -24 | ✓ Score: -24 | ✓ Score: INT_MIN* |
| Gap-affine-2p | ✓ Score: -24 | ✓ Score: -24 | ✓ Score: -24 | ✓ Score: INT_MIN* |

*CIGAR is valid, only score retrieval is affected

## Recommendations

1. **For bi-WFA with ultralow memory**, use:
   ```rust
   let aligner = AffineWavefronts::new_ultralow();
   // or
   let aligner = AffineWavefronts::with_penalties_affine2p_and_memory_mode(
       0, 4, 6, 2, 12, 1,
       MemoryMode::Ultralow
   );
   ```

2. **For score validation**, when using Ultralow mode:
   - Rely on CIGAR for alignment quality
   - Consider the score as "not available" when it's INT_MIN
   - Future work: Implement proper score retrieval from bi-aligner

3. **For maximum flexibility**, use the builder pattern:
   ```rust
   let aligner = AffineWavefrontsBuilder::new()
       .penalties(0, 4, 6, 2)
       .dual_affine_penalties(12, 1)
       .memory_mode(MemoryMode::Ultralow)
       .alignment_scope(AlignmentScope::Alignment)
       .heuristic(HeuristicStrategy::None)
       .build();
   ```

## Next Steps

1. Investigate proper score retrieval from bi-aligner in WFA2-lib
2. Consider adding validation to prevent `set_memory_mode()` after creation
3. Add similar support for Edit and Indel distance metrics
4. Document the INT_MIN score behavior in API docs