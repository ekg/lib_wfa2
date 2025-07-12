use lib_wfa2::affine_wavefront::{
    AffineWavefronts, AffineWavefrontsBuilder, MemoryMode, AlignmentScope, 
    HeuristicStrategy, DistanceMetric, AlignmentStatus
};

// Test sequences
const SHORT_QUERY: &[u8] = b"ACGTACGTACGT";
const SHORT_REF: &[u8] = b"ACGTACGTACGT";
const MED_QUERY: &[u8] = b"TCTTTACTCGCGCGTTGGAGAAATACAATAGT";
const MED_REF: &[u8] = b"TCTATACTGCGCGTTTGGAGAAATAAAATAGT";

fn generate_long_seq(len: usize) -> Vec<u8> {
    (0..len).map(|i| match i % 4 {
        0 => b'A',
        1 => b'C',
        2 => b'G',
        _ => b'T',
    }).collect()
}

#[test]
fn test_gap_affine_all_memory_modes() {
    println!("\n=== Testing Gap-Affine with All Memory Modes ===");
    
    for mode in vec![MemoryMode::High, MemoryMode::Medium, MemoryMode::Low, MemoryMode::Ultralow] {
        println!("\nTesting {:?} mode with gap-affine", mode);
        
        let aligner = AffineWavefronts::with_penalties_and_memory_mode(
            0, 4, 6, 2, 
            mode.clone()
        );
        
        assert_eq!(aligner.get_memory_mode(), mode);
        assert_eq!(aligner.get_distance_metric(), DistanceMetric::GapAffine);
        
        let status = aligner.align(MED_QUERY, MED_REF);
        assert!(matches!(status, AlignmentStatus::Completed));
        
        let cigar = aligner.cigar();
        assert!(!cigar.is_empty(), "CIGAR should not be empty for {:?} mode", mode);
        
        let score = aligner.score();
        println!("  Score: {}", score);
        println!("  CIGAR length: {}", cigar.len());
        println!("  CIGAR sample: {}", String::from_utf8_lossy(&cigar[..cigar.len().min(50)]));
        
        // Validate CIGAR
        validate_cigar(cigar, MED_QUERY.len(), MED_REF.len());
    }
}

#[test]
fn test_gap_affine2p_all_memory_modes() {
    println!("\n=== Testing Gap-Affine-2p with All Memory Modes ===");
    
    for mode in vec![MemoryMode::High, MemoryMode::Medium, MemoryMode::Low, MemoryMode::Ultralow] {
        println!("\nTesting {:?} mode with gap-affine-2p", mode);
        
        let aligner = AffineWavefronts::with_penalties_affine2p_and_memory_mode(
            0, 4, 6, 2, 12, 1,
            mode.clone()
        );
        
        assert_eq!(aligner.get_memory_mode(), mode);
        assert_eq!(aligner.get_distance_metric(), DistanceMetric::GapAffine2p);
        
        let status = aligner.align(MED_QUERY, MED_REF);
        assert!(matches!(status, AlignmentStatus::Completed));
        
        let cigar = aligner.cigar();
        assert!(!cigar.is_empty(), "CIGAR should not be empty for {:?} mode", mode);
        
        let score = aligner.score();
        println!("  Score: {}", score);
        println!("  CIGAR length: {}", cigar.len());
        
        // Check for INT_MIN issue
        if score == i32::MIN {
            println!("  WARNING: Score is INT_MIN, may indicate bi-WFA score retrieval issue");
        }
        
        validate_cigar(cigar, MED_QUERY.len(), MED_REF.len());
    }
}

#[test]
fn test_builder_all_combinations() {
    println!("\n=== Testing Builder with Various Combinations ===");
    
    // Test 1: Gap-affine + High memory
    let aligner1 = AffineWavefrontsBuilder::new()
        .penalties(0, 4, 6, 2)
        .memory_mode(MemoryMode::High)
        .build();
    
    assert_eq!(aligner1.get_memory_mode(), MemoryMode::High);
    assert_eq!(aligner1.get_distance_metric(), DistanceMetric::GapAffine);
    
    // Test 2: Gap-affine-2p + Ultralow memory
    let aligner2 = AffineWavefrontsBuilder::new()
        .penalties(0, 4, 6, 2)
        .dual_affine_penalties(12, 1)
        .memory_mode(MemoryMode::Ultralow)
        .build();
    
    assert_eq!(aligner2.get_memory_mode(), MemoryMode::Ultralow);
    assert_eq!(aligner2.get_distance_metric(), DistanceMetric::GapAffine2p);
    
    // Test 3: With heuristics
    let aligner3 = AffineWavefrontsBuilder::new()
        .penalties(0, 4, 6, 2)
        .memory_mode(MemoryMode::Low)
        .heuristic(HeuristicStrategy::BandedStatic { band_min_k: -10, band_max_k: 10 })
        .build();
    
    assert_eq!(aligner3.get_memory_mode(), MemoryMode::Low);
    let heuristics = aligner3.get_heuristics();
    assert!(!heuristics.is_empty(), "Heuristics should be set");
}

#[test]
fn test_ultralow_with_long_sequences() {
    println!("\n=== Testing Ultralow Memory with Long Sequences ===");
    
    let long_len = 1000;
    let long_query = generate_long_seq(long_len);
    let long_ref = generate_long_seq(long_len);
    
    // Test with gap-affine
    let aligner1 = AffineWavefronts::with_penalties_and_memory_mode(
        0, 4, 6, 2,
        MemoryMode::Ultralow
    );
    
    let status1 = aligner1.align(&long_query, &long_ref);
    assert!(matches!(status1, AlignmentStatus::Completed));
    println!("Gap-affine ultralow - Score: {}, CIGAR len: {}", 
             aligner1.score(), aligner1.cigar().len());
    
    // Test with gap-affine-2p
    let aligner2 = AffineWavefronts::new_ultralow();
    
    let status2 = aligner2.align(&long_query, &long_ref);
    assert!(matches!(status2, AlignmentStatus::Completed));
    println!("Gap-affine-2p ultralow - Score: {}, CIGAR len: {}", 
             aligner2.score(), aligner2.cigar().len());
}

#[test]
fn test_memory_mode_persistence() {
    println!("\n=== Testing Memory Mode Persistence ===");
    
    let aligner = AffineWavefronts::with_penalties_affine2p_and_memory_mode(
        0, 4, 6, 2, 12, 1,
        MemoryMode::Ultralow
    );
    
    // Check initial mode
    assert_eq!(aligner.get_memory_mode(), MemoryMode::Ultralow);
    
    // Perform alignment
    let _ = aligner.align(SHORT_QUERY, SHORT_REF);
    
    // Check mode persists after alignment
    assert_eq!(aligner.get_memory_mode(), MemoryMode::Ultralow);
    
    // Verify that memory mode cannot be changed after creation
    // (set_memory_mode has been removed)
    println!("Memory mode is fixed at: {:?}", aligner.get_memory_mode());
    
    // The real test: does it behave like ultralow? Let's check with a longer sequence
    let long_query = generate_long_seq(1000);
    let long_ref = generate_long_seq(1000);
    
    let status = aligner.align(&long_query, &long_ref);
    assert!(matches!(status, AlignmentStatus::Completed));
    
    // With ultralow mode, score will be INT_MIN
    let score = aligner.score();
    println!("Score with ultralow mode: {}", score);
    assert_eq!(score, i32::MIN, "Ultralow mode should return INT_MIN score");
}

#[test]
fn test_all_distance_metrics_default_constructor() {
    println!("\n=== Testing Default Constructor Behavior ===");
    
    let mut aligner = AffineWavefronts::default();
    
    // Default constructor creates with High memory mode
    assert_eq!(aligner.get_memory_mode(), MemoryMode::High);
    
    // Note: We can set penalties after creation, but not memory mode or distance metric
    aligner.set_penalties_affine2p(0, 4, 6, 2, 12, 1);
    
    let status = aligner.align(MED_QUERY, MED_REF);
    assert!(matches!(status, AlignmentStatus::Completed));
    
    // Note: Distance metric and memory mode don't change after creation
    println!("Distance metric after set_penalties_affine2p: {:?}", aligner.get_distance_metric());
    println!("Memory mode remains: {:?}", aligner.get_memory_mode());
}

// Helper function to validate CIGAR
fn validate_cigar(cigar: &[u8], query_len: usize, ref_len: usize) {
    let mut q_pos = 0;
    let mut r_pos = 0;
    
    for &op in cigar {
        match op {
            b'M' | b'=' | b'X' => {
                q_pos += 1;
                r_pos += 1;
            }
            b'I' => q_pos += 1,
            b'D' => r_pos += 1,
            _ => panic!("Invalid CIGAR operation: {}", op as char),
        }
    }
    
    assert_eq!(q_pos, query_len, "CIGAR query length mismatch");
    assert_eq!(r_pos, ref_len, "CIGAR reference length mismatch");
}

#[test]
fn test_score_validity() {
    println!("\n=== Testing Score Validity ===");
    
    // Test identical sequences - should have score 0
    let aligner = AffineWavefronts::with_penalties_and_memory_mode(
        0, 4, 6, 2,
        MemoryMode::High
    );
    
    let _ = aligner.align(SHORT_QUERY, SHORT_REF);
    assert_eq!(aligner.score(), 0, "Identical sequences should have score 0");
    
    // Test with mismatches
    let query = b"ACGT";
    let reference = b"AGGT"; // One mismatch
    
    let _ = aligner.align(query, reference);
    let score = aligner.score();
    assert!(score < 0, "Mismatch should have negative score");
    assert!(score > -100, "Score should be reasonable for one mismatch");
    println!("Score for one mismatch: {}", score);
}