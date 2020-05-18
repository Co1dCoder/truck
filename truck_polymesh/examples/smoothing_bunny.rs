extern crate truck_io as io;
use truck_polymesh::*;

fn main() {
    let mesh = io::obj::read(&"tests/data/bunny.obj").unwrap();
    let mut handler = MeshHandler::new(mesh);
    handler.add_smooth_normal(std::f64::consts::PI / 3.0);

    let file = std::fs::File::create("smooth_bunny.obj").unwrap();
    io::obj::write(&handler.into(), file).unwrap();
}
