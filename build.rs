fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    #[cfg(feature = "sp1-soldering")]
    {
        println!("cargo:rerun-if-env-changed=SP1_ELF_sp1-soldering-guest");
        println!("cargo:rerun-if-env-changed=SP1_SOLDERING_ELF_PATH");
    }
}
