Requires Rust nightly, all LLVM packages (or minimally `llvm` and `polly`), `zstd` and `libz` libraries, and (probably) a manual export of `LLVM_SYS_$(MAJOR)_PREFIX` pointing to LLVM.

Tested on LLVM version as per `compiler/Cargo.toml`