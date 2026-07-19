fn main() {
    let ct = "en mwkw xr wlahg wlv igapw xr w zgba pfsi vejd qkwql inzyq";
    
    // Test the Vigenere decoder directly
    let vigenere = sailii::decoders::Decoder::<sailii::decoders::vigenere_decoder::VigenereDecoder>::new();
    let checker = sailii::checkers::CheckerTypes::Athena(
        sailii::checkers::Checker::<sailii::checkers::Athena>::new()
    );
    
    println!("Calling Vigenere decoder directly...");
    let result = vigenere.crack(ct, &checker);
    
    println!("success: {}", result.success);
    println!("key: {:?}", result.key);
    println!("decoder: {}", result.decoder);
    if let Some(texts) = &result.unencrypted_text {
        for t in texts {
            println!("plaintext: {}", t);
        }
    }
    
    // Also test the English checker directly
    let english = sailii::checkers::CheckerTypes::English(
        sailii::checkers::Checker::<sailii::checkers::english::EnglishChecker>::new()
    );
    let check_result = english.check_text("bn jant xo aoxhd aos idest xo a cdbx tipi sima qhati ikdbn");
    println!("\nEnglish checker for DADWX output:");
    println!("  is_identified: {}", check_result.is_identified);
    println!("  match_ratio: {}", check_result.match_ratio);
}
