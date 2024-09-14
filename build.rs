// build.rs

use std::{
    env,
    error,
    time::{SystemTime, UNIX_EPOCH}
};

fn main() -> Result<(), Box<dyn error::Error>>
{
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    println!("cargo:rerun-if-changed={}", current_time);

    if let Ok(rustflags) = env::var("RUSTFLAGS") {
        unsafe {
            env::set_var(
                "RUSTFLAGS",
                format!(
                    "{} -C target-cpu=generic -C target-feature=+crt-static -C codegen-units=1 -C \
                     relocation-model=pic -C link-args=-s -C panic=abort",
                    rustflags
                )
            );
        }
    }
    else {
        unsafe {
            env::set_var(
                "RUSTFLAGS",
                "-C target-cpu=generic -C target-feature=+crt-static -C codegen-units=1 -C \
                 relocation-model=pic -C link-args=-s -C panic=abort"
            );
        }
    }

    println!("cargo:rustc-link-arg=-flto=fat");

    println!("cargo:rustc-link-arg=-fPIC");

    println!("cargo:rustc-link-arg=-s");

    println!("cargo:rustc-link-arg=-Wl,--gc-sections");

    Ok(())
}
