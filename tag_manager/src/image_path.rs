use std::{ops::Deref, path::PathBuf};

use uuid::Uuid;

pub struct ImagePath{
    pub path : PathBuf,
}

impl ImagePath{
    pub fn to_discarded(discarded_path : &PathBuf, uuid : Uuid)-> Self{
        let mut t = discarded_path.join(uuid.to_string());
        t.set_extension(".png");

        Self{
            path: t
        }
    }
    pub fn to_destination(storage_path: &PathBuf, id: u32) -> Self{
        let mut t = storage_path.join(id.to_string());
        t.set_extension(".png");

        Self{
            path: t
        }
    }
}


impl Deref for ImagePath{
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}
