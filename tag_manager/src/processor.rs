use anyhow::{Result, anyhow};
use futures::{StreamExt, stream};
use image::DynamicImage;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::PathBuf;

use crate::{
    database::Database,
    image_path::{to_discarded, to_storage, to_storage_thumbnail, to_video},
    tag_fetcher::{self, ImageFetcherError},
};

pub async fn process_images(database: &(impl Database + Clone)) -> Result<()> {
    let files = get_image_paths(&database.config().import_path)?;

    stream::iter(files)
        .map(|path| {
            let db = database.clone();
            async move {
                let extension = path
                    .extension()
                    .map(|x| x.to_str().unwrap())
                    .unwrap_or("png");
                if ["webm", "mov", "mp4", "flv", "avi"].contains(&extension) {
                    process_video(&path, extension).await
                } else {
                    match process_image(&db, &path).await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            if let Some(api_error) = e.downcast_ref::<ImageFetcherError>() {
                                println!("API failure: {api_error}");
                            } else {
                                println!(
                                    "Something went wrong processing file: {path:?} with error: {e:?}",
                                );
                                let new_path = to_discarded();
                                if let Err(e) = tokio::fs::rename(path, new_path).await {
                                    println!("Could not move errored file: {e}");
                                }
                            }
                            Err::<(), _>(e)
                        }
                    }
                }
            }
        })
        .buffer_unordered(12)
        .collect::<Vec<_>>()
        .await;

    thumbnail_images(database).await?;
    Ok(())
}

fn get_image_paths(import_path: &PathBuf) -> Result<Vec<PathBuf>> {
    Ok(std::fs::read_dir(import_path)?
        .filter_map(Result::ok)
        .map(|x| x.path())
        .collect())
}

async fn process_video(path: &PathBuf, extension: &str) -> Result<()> {
    let new_path = to_video(extension);

    tokio::fs::rename(path, new_path).await?;

    Ok(())
}

async fn process_image(database: &impl Database, path: &PathBuf) -> Result<()> {
    let path_n = path.clone();
    let image = tokio::task::spawn_blocking(move || {
        image::io::Reader::open(&path_n).map(|ok| {
            ok.with_guessed_format().map(|mut okk| {
                okk.no_limits();
                okk.decode()
            })
        })
    })
    .await????;
    let hash: [u8; 8] = imagehash::average_hash(&image)
        .to_bytes()
        .try_into()
        .unwrap();
    let exists = database.check_hash(&hash).await?;
    if exists {
        return Err(anyhow!("File duplicate."));
    }
    let tags = tag_fetcher::fetch_tags(&image).await?;
    let id = database.save_image(&hash, &tags).await?;

    let new_path = to_storage(id);
    let image_copy = image.clone();
    if let Err(e) = tokio::task::spawn_blocking(move || {
        image.save_with_format(new_path, image::ImageFormat::Png)
    })
    .await?
    {
        println!("Error saving file!!! bad!!!!, remove {id}, from db, {e}",);
    }

    thumbnail_image_from_file(database, id, image_copy).await?;

    tokio::fs::remove_file(path).await?;
    Ok(())
}

async fn thumbnail_images(database: &impl Database) -> Result<()> {
    let non_processed_images = database.get_non_thumbnailed_images().await?;

    stream::iter(non_processed_images)
        .map(|image| {
            let db = database.clone();
            async move {
                match thumbnail_image(db, image).await{
                    Ok(_) => Ok(()),
                    Err(e) => {
                        println!("Something went wrong processing file: {image}, with error: {e}");
                        Err(e)
                    }
                }
            }
        })
        .buffer_unordered(12)
        .collect::<Vec<_>>()
        .await;

    Ok(())
}
async fn thumbnail_image(database: &impl Database, image_id: u32) -> Result<()> {
    let path = to_storage(image_id);
    let image = tokio::task::spawn_blocking(move || {
        image::io::Reader::open(&path).map(|ok| {
            ok.with_guessed_format().map(|mut okk| {
                okk.no_limits();
                okk.decode()
            })
        })
    })
    .await????;

    thumbnail_image_from_file(database, image_id, image).await
}

async fn thumbnail_image_from_file(
    database: &impl Database,
    image_id: u32,
    image: DynamicImage,
) -> Result<()> {
    let thumbnail = image.resize(
        database.config().thumbnail_size,
        database.config().thumbnail_size,
        image::imageops::FilterType::Lanczos3,
    );
    let output_path = to_storage_thumbnail(image_id);
    let mut output = std::fs::File::create(output_path)?;
    thumbnail.write_to(&mut output, image::ImageOutputFormat::Jpeg(60))?;
    database.write_thumbnail(image_id).await
}
