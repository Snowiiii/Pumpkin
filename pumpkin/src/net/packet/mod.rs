mod config;
mod handshake;
mod login;
mod play;
mod status;

fn is_valid_player_name(name: &str) -> bool {
    name.len() <= 16 && name.chars().all(|c| c > 32u8 as char && c < 127u8 as char)
}
