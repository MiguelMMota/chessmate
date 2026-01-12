// Pure Rust game logic modules - NO Godot dependencies
pub mod ai;
mod cards;
pub mod game;
pub mod networking; // Public for server binary

// FFI layer for external clients (Godot, web, etc.)
pub mod ffi;

// Godot-specific bridge (only compiled when godot feature is enabled)
#[cfg(feature = "godot")]
mod godot_bridge;

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic() {
        assert_eq!(2 + 2, 4);
    }
}
