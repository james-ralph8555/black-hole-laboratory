//! The simulation crate handles the heavy lifting of general relativity, 
//! solving the geodesic equations to determine how light travels.

/// This is a placeholder.
///
/// In the future, this crate will define the spacetime metric in Kerr-Schild coordinates,
/// calculate the Christoffel symbols, and provide a function to integrate the geodesic
/// equation.
pub fn get_placeholder_string() -> &'static str {
    "Hello from the simulation crate!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(get_placeholder_string(), "Hello from the simulation crate!");
    }
}
