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
                    "{} -C target-cpu=generic -C target-feature=+crt-static -C \
                     relocation-model=pic",
                    rustflags
                )
            );
        }
    }
    else {
        unsafe {
            env::set_var(
                "RUSTFLAGS",
                "-C target-cpu=generic -C target-feature=+crt-static relocation-model=pic"
            );
        }
    }

    println!("cargo:rustc-link-arg=-s");

    println!("cargo:rustc-link-arg=-Wl,--gc-sections");

    Ok(())
}
