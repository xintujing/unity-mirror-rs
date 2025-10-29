use serde::Deserialize;

pub mod metadata_network_manager_ext;
pub mod metadata_player_ext;
mod metadata_sync_materials;
mod metadata_sync_virtual_cut_card_tracking_camera;
mod metadata_sync_virtual_shuffle_tracking_camera;
mod metadata_sync_virtual_start_cameraa;
mod metadata_character_controller;
pub mod metadata_poker_controller;
pub mod network_manager_ext_status;

#[derive(Deserialize)]
pub struct PlayAlgorithmSycGamePlayer {}