use lib_wfa2::affine_wavefront::{AffineWavefronts, AffineWavefrontsBuilder, MemoryMode, AlignmentScope, HeuristicStrategy};

pub fn main() {
    println!("Example: Ultra-low memory bi-WFA alignment\n");

    // Method 1: Quick constructor for ultralow memory
    println!("Method 1: Using new_ultralow()");
    let aligner = AffineWavefronts::new_ultralow();
    
    let pattern = b"TCTTTACTCGCGCGTTGGAGAAATACAATAGT";
    let text = b"TCTATACTGCGCGTTTGGAGAAATAAAATAGT";
    
    let status = aligner.align(pattern, text);
    
    println!("Pattern: {}", String::from_utf8_lossy(pattern));
    println!("Text:    {}", String::from_utf8_lossy(text));
    println!("Status:  {:?}", status);
    println!("Score:   {}", aligner.score());
    println!("CIGAR:   {}", String::from_utf8_lossy(aligner.cigar()));
    println!();

    // Method 2: Using specific constructor with memory mode
    println!("Method 2: Using with_penalties_affine2p_and_memory_mode()");
    let aligner2 = AffineWavefronts::with_penalties_affine2p_and_memory_mode(
        0,   // match
        6,   // mismatch
        4,   // gap opening 1
        2,   // gap extension 1
        12,  // gap opening 2
        1,   // gap extension 2
        MemoryMode::Ultralow,
    );
    
    let status2 = aligner2.align(pattern, text);
    println!("Score with different penalties: {}", aligner2.score());
    println!();

    // Method 3: Using builder for maximum control
    println!("Method 3: Using builder pattern");
    let aligner3 = AffineWavefrontsBuilder::new()
        .penalties(0, 4, 6, 2)           // match, mismatch, gap_open, gap_ext
        .dual_affine_penalties(12, 1)    // gap_open2, gap_ext2
        .memory_mode(MemoryMode::Ultralow)
        .alignment_scope(AlignmentScope::Alignment)
        .heuristic(HeuristicStrategy::None)
        .build();
    
    let status3 = aligner3.align(pattern, text);
    println!("Score with builder: {}", aligner3.score());
    println!("Memory mode verified: {:?}", aligner3.get_memory_mode());
}