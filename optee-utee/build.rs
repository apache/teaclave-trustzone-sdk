use std::fs;

/// Outputs unstable #[feature = "foo"] iff the rustc version is older than $version
macro_rules! maybe_feat {
    ($out:expr, $feat:literal, $version:literal) => {
        {
            let filename = $out.join(concat!($feat, ".rs"));
            
            let s = if version_check::is_min_version($version).unwrap_or(false) {
                String::new()
            } else {
                format!("#![feature = \"{}\"]\n", $feat)
            };
            fs::write(filename, s).expect("failed to write to {filename:?}");
        }
    }
}

fn main() {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("infallible"));

    // The custom patched std version is currently on 1.80. When we upgrade, we should
    // bump the MSRV accordingly and remove any of these features that are stablized.
    maybe_feat!(out, "error_in_core", "1.81.0");
}


