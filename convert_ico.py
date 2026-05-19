import sys
from PIL import Image
img = Image.open(sys.argv[1])
# PIL requires RGBA to save with transparency properly
if img.mode != 'RGBA':
    img = img.convert('RGBA')
# ICO usually contains multiple sizes, but a 256x256 is great.
icon_sizes = [(16,16), (32, 32), (48, 48), (64,64), (128, 128), (256, 256)]
img.save(sys.argv[2], format='ICO', sizes=icon_sizes)
