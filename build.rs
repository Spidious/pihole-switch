extern crate embed_resource;
fn main() {
    // Compile resources.rc to embed the .ico file
    embed_resource::compile("resources/resources.rc", embed_resource::NONE);
}