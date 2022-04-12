#![deny(missing_docs)]
#![deny(missing_doc_code_examples)]
//!
//! Adding docs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().build_client(false).protoc_arg("--experimental_allow_proto3_optional").compile(
        &[
            "proto/minknow_api/minion_device.proto",
            "proto/minknow_api/data.proto",
            "proto/minknow_api/protocol.proto",
            "proto/minknow_api/statistics.proto",
            "proto/minknow_api/acquisition.proto",
            "proto/minknow_api/manager.proto",
            "proto/minknow_api/protocol_settings.proto",
            "proto/minknow_api/basecaller.proto",
            "proto/minknow_api/analysis_configuration.proto",
            "proto/minknow_api/promethion_device.proto",
            "proto/minknow_api/instance.proto",
            "proto/minknow_api/log.proto",
            "proto/minknow_api/keystore.proto",
            "proto/minknow_api/rpc_options.proto",
            "proto/minknow_api/device.proto",
        ],
        &["proto/"],
    )?;
    Ok(())
}
