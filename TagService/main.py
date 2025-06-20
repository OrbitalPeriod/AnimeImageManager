from fastapi import FastAPI, UploadFile, File
from wdtagger import Tagger
from PIL import Image
import io

app = FastAPI()
tagger = Tagger()

@app.post("/tag/")
async def tag(file: UploadFile = File(...)):
    if not file.filename.lower().endswith(('.png', '.jpg', '.jpeg', '.webp')):
        return {"error": "Unsupported file type. Please upload an image."}
    try:
        image = Image.open(io.BytesIO(await file.read()))
        tags = tagger.tag(image)
        return TagData.from_tags(tags)
    except Exception as e:
        return {"error": str(e)}

class TagData:
    def __init__(self, character_tags, general_tags, rating):
        self.character_tags = character_tags
        self.general_tags = general_tags
        self.rating = rating
    def from_tags(tags):
        return TagData(
            character_tags=tags.character_tags,
            general_tags=tags.general_tags,
            rating=tags.rating
        )
