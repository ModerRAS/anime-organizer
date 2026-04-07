fn main() {
    #[cfg(feature = "clouddrive")]
    {
        tonic_build::configure()
            .compile_well_known_types(false)
            .build_server(false)
            .compile_protos(&["proto/clouddrive.proto"], &["proto"])
            .expect("failed to compile clouddrive.proto");
    }
}
