//! Prints the model represented by a structural fuzz input.

use quote::ToTokens;

fn main() -> std::io::Result<()> {
    let path = std::env::args_os().nth(1).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "expected a corpus input path")
    })?;
    let bytes = std::fs::read(path)?;
    let model = diesel_builders_derive_fuzz::model_input::generate_model(&bytes);
    println!("{}", model.into_token_stream());
    Ok(())
}
