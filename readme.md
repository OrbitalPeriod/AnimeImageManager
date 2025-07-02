# Env variables:
DATABASE_URL: Database connection URL 
PIXIV_REFRESH_TOKEN: Pixiv refresh token 
PIXIV_USR_ID: Pixiv usr id
IMPORT_DIR(Optional, Defaults to /Images/Import): Path pointing to import DIR
STORAGE_DIR(Optional, Defaults to /Images/Storage): Path pointing to the storage DIR
DISCARDED_DIR (Optional, Defaults to /Images/Disard): Path pointing to the discard dir
VIDEO_DIR (Optional, Defaults to /Images/Videos): Path pointing to the video dir
API_ADDRESS (Optional, defaults to 0.0.0.0): address of the API endpoint
API_PORT (Optional, defaults to 8080): Port of the API endpoint
IMAGE_URL_PREFIX (Defaults to localhost): Prefix for the image to serve
TAGGSERVICE_URL (Dont change, defaults to 127.0.0.1:8000): Point manager uses to tag images

# Mounting points 
<local>:/Images, such that /Images/Import exists, etc...
