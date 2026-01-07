// core/build.rs

use std::env;

fn main() {
    // Učitaj .env datoteku iz korijena projekta (gdje je Cargo.toml)
    dotenvy::dotenv().ok();
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=build.rs");

    // 1. VALTER_PROVIDER
    let provider = env::var("VALTER_PROVIDER").unwrap_or_else(|_| "gemini".to_string());
    println!("cargo:rustc-env=VALTER_PROVIDER={}", provider);

    // 2. VALTER_GEMINI_API_KEY (Obavezan ako je provider gemini)
    if provider == "gemini" {
        let api_key = env::var("VALTER_GEMINI_API_KEY")
            .expect("Greška: VALTER_GEMINI_API_KEY mora biti postavljen u .env datoteci za build.");
        println!("cargo:rustc-env=VALTER_GEMINI_API_KEY={}", api_key);
    } else {
        // Ako provider nije gemini, ugradi prazan string
        println!("cargo:rustc-env=VALTER_GEMINI_API_KEY=");
    }

    // 3. VALTER_MODEL
    let model = env::var("VALTER_MODEL").unwrap_or_else(|_| "gemini-2.5-flash-lite".to_string());
    println!("cargo:rustc-env=VALTER_MODEL={}", model);

    // 4. VALTER_RPM
    let rpm = env::var("VALTER_RPM").unwrap_or_else(|_| "30".to_string());
    println!("cargo:rustc-env=VALTER_RPM={}", rpm);

    // 5. VALTER_SEARCH_API_KEY (Nije obavezan)
    let search_api_key = env::var("VALTER_SEARCH_API_KEY").unwrap_or_default();
    println!("cargo:rustc-env=VALTER_SEARCH_API_KEY={}", search_api_key);

    // 6. VALTER_SEARCH_CX (Nije obavezan)
    let search_cx = env::var("VALTER_SEARCH_CX").unwrap_or_default();
    println!("cargo:rustc-env=VALTER_SEARCH_CX={}", search_cx);

    // 7. ENV_VARIABLES_HIERARCHY_ENABLE_OVERRIDES_FROM_SYSTEM_DURING_RUNTIME (Nije obavezan)
    // --- ISPRAVLJEN NAZIV VARIJABLE ---
    let enable_overrides =
        env::var("ENV_VARIABLES_HIERARCHY_ENABLE_OVERRIDES_FROM_SYSTEM_DURING_RUNTIME")
            .unwrap_or_else(|_| "false".to_string());
    println!(
        "cargo:rustc-env=ENV_VARIABLES_HIERARCHY_ENABLE_OVERRIDES_FROM_SYSTEM_DURING_RUNTIME={}",
        enable_overrides
    );
}
