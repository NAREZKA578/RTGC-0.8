//! Assets module for RTGC-0.7

pub mod loader;
pub mod asset_loader;
pub mod vehicle_loader;

pub use loader::{AssetLoader, AssetHandle, AssetData, AssetType, AssetMetadata, LoaderConfig, AssetLoadError};
pub use asset_loader::{Asset, AssetManager, VehicleAsset, VehiclePreset, GameObjectAsset};
pub use vehicle_loader::{VehicleLoader, VehicleDefinition, VehicleMetadata, VehicleLoadError};
