# Scarlet
## Colors and color spaces made simple, for Rustaceans.

**Note: This is still in development and is not ready for use. Stay tuned for version 1.0 before
June 2018. This README, in its current state, describes what Scarlet will be, not what it is.**

Humans and computers don't view color the same way, and so color descriptions like RGB don't
effectively describe colors in the way that humans perceive them and vice versa. Image processing
and other disciplines require the ability to interconvert and work with colors in a way that
respects both how colors are displayed and how they are perceived by the human eye. Scarlet makes
this not only possible, but simple and convenient. With Scarlet, you can:
 * Interconvert between different color representations
 * Work *with* colors in one representation *using* the concepts of a different one. For example,
   you can use a model of color luminance that accurately models human vision without leaving RGB,
   or modify a color's hue in CIELAB.
 * Mix and average colors accurately, without kludges or results that look wrong.
 * Create perceptually-uniform color scales, colormaps, and gradients suitable for publication-quality visuals that don't
   mislead the viewer.
 * Convert a color to grayscale accurately and precisely.
 * Accurately determine how far apart colors are to humans
 * And more!
 
 



