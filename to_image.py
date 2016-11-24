#!/usr/bin/env python2

import numpy
from scipy.signal import firls, lfilter, resample_poly
from scipy.io import wavfile
import scipy.misc
import matplotlib.pyplot as plt


f_carrier = 2400.0

f_lim = 4160.0
trans_width = 500.0

f_samp, raw = wavfile.read("demod.wav")

f_samp *= 1.0
p_samp = 1.0/f_samp
duration = p_samp * raw.size
samples_per_line = 2080.0

resampled = resample_poly(raw, 13, 150)


missing_elements = int(numpy.ceil(resampled.size / samples_per_line) * samples_per_line) - resampled.size
padded = numpy.append(resampled, [0] * missing_elements)


image = numpy.reshape(padded, (padded.size / samples_per_line, samples_per_line))

scipy.misc.toimage(image).save("noaa.png")
