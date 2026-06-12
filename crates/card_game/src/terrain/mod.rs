mod generated {
    include!(concat!(env!("OUT_DIR"), "/terrain.rs"));
}

pub use generated::tileset;
