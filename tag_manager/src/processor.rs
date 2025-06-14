use std::{error::Error, fs::DirEntry};

use futures::{StreamExt, stream};
use uuid::Uuid;

use crate::{database::Database, image_path::ImagePath, tag_fetcher};

pub async fn process_images(
    database: impl Database + Clone,
) -> Result<(), Box<dyn Error>> {
    let files: Vec<_> = std::fs::read_dir(database.config().import_path.clone())
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    stream::iter(files)
        .map(|file| {
            let db = database.clone();
            async move {
                match process_image(&db, &file).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        println!(
                            "Something went wrong processing file: {:?} with error: {:?}",
                            file, e
                        );
                        let uid = Uuid::new_v4();
                        let new_path = ImagePath::to_discarded(&db.config().discarded_path, uid);
                        if let Err(e) = tokio::fs::rename(file.path(), new_path.path).await {
                            println!("Could not move errored file: {}", e);
                        }
                        Err::<(), _>(e)
                    }
                }
            }
        })
        .buffer_unordered(4) // Limit concurrency to 4 tasks
        .collect::<Vec<_>>() // Wait for all tasks to finish
        .await;
    Ok(())
}

async fn process_image(
    database: &impl Database,
    file: &DirEntry,
) -> Result<u32, Box<dyn Error + Send + Sync>> {
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
    if let Err(e) = image.save_with_format(new_path, image::ImageFormat::Png) {
        println!(
            "Error saving file!!! bad!!!!, remove {}, from db, {}",
            id, e
        );
    }

    tokio::fs::remove_file(path).await?;
    Ok(id)
}
