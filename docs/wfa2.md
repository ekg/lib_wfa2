# WFA2lib C++ API Usage in WFMash

This document describes how WFMash integrates and uses the WFA2lib (Wavefront Alignment Algorithm) C++ API for sequence alignment.

## Overview

WFMash uses WFA2lib for performing pairwise sequence alignments through the `wflign` component. The integration is primarily through the `wfa::WFAligner` classes, with two main variants being used:

1. `wfa::WFAlignerGapAffine2Pieces` - For alignments with dual affine gap penalties
2. `wfa::WFAlignerGapAffine` - For alignments with single affine gap penalties

## Key API Components

### 1. Aligner Classes

```cpp
// Main aligner for dual affine gap penalties
wfa::WFAlignerGapAffine2Pieces wf_aligner(
    match_score,              // typically 0
    mismatch_score,          // e.g., 4
    gap_opening1_score,      // e.g., 6
    gap_extension1_score,    // e.g., 1
    gap_opening2_score,      // e.g., 13
    gap_extension2_score,    // e.g., 1
    wfa::WFAligner::Alignment,     // alignment mode
    wfa::WFAligner::MemoryUltralow // memory mode
);
```

### 2. Target/Query Order

**CRITICAL**: WFA2lib uses a specific parameter order that may be counterintuitive:

```cpp
// The alignment function signature:
int alignEnd2End(
    const char* target,    // First parameter is TARGET
    int target_length,     
    const char* query,     // Second parameter is QUERY
    int query_length
);
```

This order is consistently used throughout WFMash:
- **Target** (reference) comes **first**
- **Query** comes **second**

### 3. Alignment Types

WFMash uses several alignment modes:

#### a. End-to-End Alignment
```cpp
const int status = wf_aligner.alignEnd2End(target, target_length, query, query_length);
```

#### b. Semi-Global Alignment (for patching)
```cpp
// For head patching - free gaps at the beginning
const int head_status = head_aligner.alignEndsFree(
    target_str,
    target_length, 0,  // textBeginFree, textEndFree
    query_str,
    query_length, 0    // patternBeginFree, patternEndFree
);

// For tail patching - free gaps at the end
const int tail_status = tail_aligner.alignEndsFree(
    target_str,
    0, target_length,  // textBeginFree, textEndFree
    query_str,
    0, query_length    // patternBeginFree, patternEndFree
);
```

### 4. Heuristics Configuration

WFMash configures various heuristics:

```cpp
// No heuristics (exact alignment)
wf_aligner.setHeuristicNone();

// WFMash-specific heuristics
wflambda_aligner->setHeuristicWFmash(
    min_wavefront_length,     // e.g., 100
    max_distance_threshold    // e.g., computed from identity
);
```

### 5. CIGAR String Extraction

The CIGAR string is extracted using two methods:

#### a. Direct CIGAR Copy (Internal Format)
```cpp
void wflign_edit_cigar_copy(
    wfa::WFAligner& wf_aligner,
    wflign_cigar_t* const cigar_dst
) {
    char* cigar_ops;
    int cigar_length;
    wf_aligner.getAlignment(&cigar_ops, &cigar_length);
    
    // Copy to destination structure
    cigar_dst->cigar_ops = (char*)malloc(cigar_length);
    cigar_dst->begin_offset = 0;
    cigar_dst->end_offset = cigar_length;
    memcpy(cigar_dst->cigar_ops, cigar_ops, cigar_length);
}
```

#### b. String Format (for patching)
```cpp
std::string cigar_long = aligner.getAlignment();  // Returns long-form CIGAR
```

### 6. CIGAR Format Details

WFA2lib uses different CIGAR representations:

1. **Internal Format**: Character array where each character represents one operation
   - 'M' or '=' for match
   - 'X' for mismatch
   - 'I' for insertion
   - 'D' for deletion

2. **Standard Format**: Run-length encoded (e.g., "10M2I5M")
   - Converted using `wfa_alignment_to_cigar()` function
   - 'M' is typically converted to '=' in output

### 7. Score Handling

```cpp
// Get alignment score (lower is better in WFA)
int score = wf_aligner.getAlignmentScore();

// Check alignment status
if (wf_aligner.getAlignmentStatus() == WF_STATUS_ALG_COMPLETED) {
    // Alignment successful
}
```

### 8. Alignment Positions

The alignment positions are tracked in the `alignment_t` structure:

```cpp
struct alignment_t {
    int j;              // Query start position (0-based)
    int i;              // Target start position (0-based)
    int query_length;   // Aligned query length
    int target_length;  // Aligned target length
    int score;          // Alignment score
    bool ok;            // Alignment success flag
    bool is_rev;        // Reverse complement flag
    wflign_cigar_t edit_cigar;  // CIGAR operations
};
```

### 9. Memory Management

WFMash uses different memory modes for different contexts:
- `MemoryUltralow`: For main alignments
- `MemoryMed`: For patching operations
- `MemoryHigh`: For complex alignments with plotting

### 10. Penalty Structure

```cpp
typedef struct {
    int match;           // Usually 0
    int mismatch;        // e.g., 4
    int gap_opening1;    // e.g., 6
    int gap_extension1;  // e.g., 1
    int gap_opening2;    // e.g., 13
    int gap_extension2;  // e.g., 1
} wflign_penalties_t;
```

## Alignment Workflow

1. **Create aligner** with appropriate penalties and memory mode
2. **Set heuristics** (or disable them)
3. **Perform alignment** using appropriate method
4. **Check status** for successful completion
5. **Extract CIGAR** using internal copy method
6. **Convert CIGAR** to standard format if needed
7. **Process alignment** (patching, trimming, etc.)
8. **Output results** in PAF or SAM format

## Special Features

### Chain Patching
WFMash implements a sophisticated patching system to improve alignment ends:
- Erodes alignment ends to expose difficult regions
- Re-aligns with semi-global alignment
- Merges results back into main alignment

### CIGAR Swizzling
Post-processes CIGAR strings to optimize certain patterns for better biological accuracy.

### Reverse Complement Handling
WFMash handles reverse complement alignments at a higher level, not directly through WFA2lib.

## Important Notes

1. **Coordinate System**: All positions are 0-based
2. **Memory Ownership**: CIGAR strings must be properly freed after use
3. **Thread Safety**: Each thread should have its own aligner instance
4. **Score Interpretation**: WFA scores are edit distances (lower is better)
5. **Parameter Order**: Always remember target-first, query-second convention