extern crate prost_build;

fn main() {
  let proto_files = [
    "src/proto/fileformat.proto",
    "src/proto/osmformat.proto",
  ];

  for path in &proto_files {
    println!("cargo:rerun-if-changed={path}");
  }

  prost_build::compile_protos(&proto_files, &["src/proto"]).unwrap();
}