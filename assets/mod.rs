//! Assets module for RTGC-0.7

pub mod loader;
pub mod assets_module;
pub mod asset_loader;
pub mod vehicle_loader;

pub use loader::{AssetLoader, AssetHandle, AssetData, AssetType, AssetMetadata, LoaderConfig, AssetLoadError};
pub use vehicle_loader::{VehicleLoader, VehicleDefinition, VehicleMetadata, VehicleLoadError};
