#[cfg(windows)]
use winres::WindowsResource;

fn main() {
    #[cfg(windows)]
    #[allow(unused_must_use)]
    {
        WindowsResource::new().set_icon("assets/icon.ico").compile();
    }
}
