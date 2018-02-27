% Color Distance in Scarlet

A somewhat common thing to imbue colors with is a *metric*, or a way of taking distances between
them. It's very intuitive to think that blue and green are closer than black and white, or that red
and yellow are closer than red and green.

Scarlet allows us to go one step beyond intuition, and actually quantify this color difference in an
observer-independent way. This is crucial for applications that require exactitude. Is this lighting
environment really close enough to a different one to work? Is the amount of measurement error in an
experiment appropriate? Will anyone notice the difference in color between a printed and onscreen
logo?

The most tempting way of computing color difference is to treat it as the distance between 3D points
in some color system. Using common spaces like RGB, however, will lead to extremely poor
results. The reason for this is that the human eye distinguishes very differently between different
types of colors. For example, in low-light conditions we can detect differences in lightness more
easily, and we're better at distinguishing blues than greens. RGB does attempt to account for this,
but it's not a substitute for a real perceptually-accurate correction for such issues.

CIELAB and other CIE spaces do much better, and in a pinch most of them provide alright metrics to
judge color difference. However, over time the CIE has found more and more idiosyncrasies in human
color perception that render these inaccurate. They have released these changes as a new color
difference function, the CIEDE2000. This is what Scarlet implements and uses for the
[`distance`](scarlet::Color::distance) and
[`visually_indistinguishable`](scarlet::Color::visually_indistinguishable) methods. You can
consult those methods' documentation for concrete examples.

To summarize: when you want to determine how different two colors look, use the `distance`
method. If you want to determine whether two colors are indistinguishable to the average human
observer, use `visually_indistinguishable`, and keep in mind that it is a conservative estimate more
likely to say that two indistinguishable colors are not than the opposite.
