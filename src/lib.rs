use godot::prelude::*;

// Module declarations
mod game;
mod ai;
mod networking;
mod cards;

// Main extension struct
struct ChessMateExtension;

#[gdextension]
unsafe impl ExtensionLibrary for ChessMateExtension {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(2 + 2, 4);
    }
}
