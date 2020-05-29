import skimage
from skimage import io as sio
from skimage.color import rgb2gray, gray2rgb

import numpy as np

im = sio.imread("data/leaves_big_oak.png") / 255

color = [int('41', 16), int('b9', 16), int('41', 16), 255]

res = np.einsum('ijk,k->ijk', im, color)
sio.imsave("result.png", res)
