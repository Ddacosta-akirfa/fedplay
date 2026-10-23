fn main() {
    glib_build_tools::compile_resources(
        &["data"],
        "data/fedplay.gresource.xml",
        "fedplay.gresource",
    );
}
