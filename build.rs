fn main() {
    #[cfg(all(windows, feature = "anifilebert"))]
    {
        // Pyke's Windows ORT packages are built with DirectML support even for CPU use.
        // When callers prefer dynamic linking via ORT_LIB_LOCATION/ORT_PREFER_DYNAMIC_LINK,
        // the final link still needs these system import libs.
        println!("cargo:rustc-link-lib=DirectML");
        println!("cargo:rustc-link-lib=dxguid");
        println!("cargo:rustc-link-lib=DXCORE");
        println!("cargo:rustc-link-lib=DXGI");
        println!("cargo:rustc-link-lib=D3D12");
    }

    #[cfg(feature = "clouddrive")]
    {
        tonic_build::configure()
            .compile_well_known_types(false)
            .build_server(false)
            .compile_protos(&["proto/clouddrive.proto"], &["proto"])
            .expect("failed to compile clouddrive.proto");
    }
}
