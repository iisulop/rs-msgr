use std::env;
use std::io::Result;

fn main() -> Result<()> {
    println!("{:#?}", env::current_dir());
    println!("{:#?}", env::var("OUT_DIR"));
    prost_build::compile_protos(&["src/types.proto"], &["src/"])?;
    Ok(())
}
