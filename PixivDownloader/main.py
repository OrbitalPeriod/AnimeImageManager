import os
import time
import asyncio
from dotenv import load_dotenv
from pixivpy3 import AppPixivAPI
from sqlalchemy import create_engine, Column, String, Integer, exists
from sqlalchemy.orm import sessionmaker, declarative_base
from uuid import uuid4
import logging

# Load environment
load_dotenv("../.env")
DATABASE_URL = os.getenv("DATABASE_URL") or Exception("DATABASE URL MUST BE GIVEN")
PIXIV_API_TOKEN = os.getenv("PIXIV_REFRESH_TOKEN") or Exception("PXIIV ACCESS TOKEN MUST BE GIVEN")
IMAGE_DIR = os.getenv("IMPORT_DIR") or "/Images/Import/"
PIXIV_USR_ID = os.getenv("PIXIV_USR_ID") or Exception("PIXIV USER ID MUST BE GIVEN")

if PIXIV_USR_ID is None:
 Exception("PIXIV USER ID MUST BE GIVEN")
if PIXIV_API_TOKEN is None:
 Exception("PXIIV ACCESS TOKEN MUST BE GIVEN")
if DATABASE_URL is None:
 Exception("DATABASE URL MUST BE GIVEN")

# SQLAlchemy setup
Base = declarative_base()

class DownloadedImage(Base):
    __tablename__ = "downloaded_pixiv_images"
    id = Column(Integer, primary_key=True)
    pixiv_id = Column(Integer)

engine = create_engine(DATABASE_URL, pool_pre_ping=True, pool_recycle=300)
Base.metadata.create_all(engine)
Session = sessionmaker(bind=engine)
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    handlers=[
        logging.FileHandler("pixiv_downloader.log"),
        logging.StreamHandler()
    ]
)

# Setup Pixiv API

def get_nondownloaded_bookmarks(session, api):
    bookmarks = api.user_bookmarks_illust(user_id = PIXIV_USR_ID, restrict="public").illusts + api.user_bookmarks_illust(user_id = PIXIV_USR_ID, restrict="private").illusts
    non_downloaded_bookmarks = [bookmark for bookmark in bookmarks
        if not session.query(
            session.query(DownloadedImage.id)
            .filter(DownloadedImage.pixiv_id == bookmark.id)
            .exists()
        ).scalar()]
    return non_downloaded_bookmarks

def download_images(api, session, bookmarks):
    for illust in bookmarks:
        if illust.meta_pages:
            for page in illust.meta_pages:
                url = page.image_urls.original
                download_image(api, url)
        else:
            download_illust(api, illust)

        session.add(DownloadedImage(pixiv_id=illust.id))
        session.commit()
        time.sleep(1)


def download_illust(api, illust):
    image_url = illust.meta_single_pageoriginal_image_url or illust.image_urls.large
    download_image(api, image_url)

def download_image(api, url):
    file_name = str(uuid4()) + ".png"
    api.download(url, IMAGE_DIR, name=file_name)

def main():
    api = AppPixivAPI()
    session = Session()


    while True:
        try:
            api.auth(refresh_token=PIXIV_API_TOKEN)
            session = Session()
        except Exception as e:
            logging.critical("API failed to refresh, shutting down")
            return;

        try:
            bookmarks = get_nondownloaded_bookmarks(session, api)      
            logging.info(f"Downloading {len(bookmarks)} images...")
            download_images(api, session, bookmarks)
        except Exception as e:
            logging.error(f"Something failed when trying to download...: {e}")
        finally:
            session.close()
            logging.info("Image Downloader sleeping...")
            time.sleep(30 * 60)

if __name__ == "__main__":
    main()
