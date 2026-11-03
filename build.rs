// Gives the Windows .exe the tree icon. Other platforms get it from the window or the app bundle.
fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().expect("could not add the icon to the .exe");
    }
}
