use bytes::Buf;
use fmmap::{MmapFile, MmapFileExt};

#[tokio::main]
async fn main() {
    let file = MmapFile::open("./osmdata/data.osm.pbf").unwrap();
    let reader = file.reader(0).unwrap();
    // 
    // let reader = pbf_reader::reader::OsmPbfReader::new(reader);
    // 
    // reader.blob_reader.for_each(|el| {});
    // reader.blob_reader.for_each(|el| {});
}
