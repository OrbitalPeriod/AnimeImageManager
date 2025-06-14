use std::{
    error::{self, Error},
    fs::{self, DirEntry},
};

use rayon::iter::{ParallelBridge, ParallelIterator};
use tokio::runtime::Handle;
use uuid::Uuid;

use crate::{
    database::Database,
    image_path::{self, ImagePath},
    tag_fetcher,
};

pub async fn process_images(database: impl Database) -> Result<(), Box<dyn Error>> {
    let files = fs::read_dir(database.config().import_path.clone()).unwrap();

    for file in files {
        let file = file.unwrap();
        if let Err(e) = process_image(&database, &file).await {
            println!(
                "Something went wrong proccessing file: {:?} with error: {:?}",
                file, e
            );
            let uid = Uuid::new_v4();
            let new_path = ImagePath::to_discarded(&database.config().discarded_path, uid);
            if let Err(e) = fs::rename(file.path(), new_path.path) {
                println!("Could not move errored file: {}", e);
            }
        }
    }

    Ok(())
}

async fn process_image(database: &impl Database, file: &DirEntry) -> Result<u32, Box<dyn Error>> {
    let path = file.path();
    let image = image::io::Reader::open(&path)?
        .with_guessed_format()?
        .decode()?;
    let hash: [u8; 8] = imagehash::average_hash(&image)
        .to_bytes()
        .try_into()
        .unwrap();
    let exists = database.check_hash(&hash).await?;
    if exists {
        return Err("File duplicate".into());
    }
    let tags = tag_fetcher::fetch_tags(&image).await?;
    let id = database.save_image(&hash, &tags).await?;

    let new_path = ImagePath::to_destination(&database.config().storage_path, id).path;
    if let Err(e) = image.save(new_path) {
        println!(
            "Error saving file!!! bad!!!!, remove {}, from db, {}",
            id, e
        );
    }

    fs::remove_file(path)?;
    Ok(id)
}

enum Destination {
    Import,
    Storage,
    Discarded,
}
