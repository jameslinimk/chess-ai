use std::io::Result;

#[cfg(windows)]
use winres::WindowsResource;

fn main() -> Result<()> {
    #[cfg(windows)]
    {
        WindowsResource::new()
            .set_icon("assets/icon.ico")
            .compile()?;
    }
    Ok(())
}
