fn main() {
    uniffi::generate_scaffolding("./src/bitcell_wallet_mobile.udl")
        .expect("Failed to generate UniFFI scaffolding");
}
