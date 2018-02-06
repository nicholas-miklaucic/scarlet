% Color Space Conversion and Gamuts

# Context
Scarlet works with *color spaces*: coordinate systems that represent color. Some color spaces are
designed to match human visual perception; others are designed to match computer display of color;
others aim to achieve a balance and easily translate both to human perception and actual display of
color. Scarlet aims to allow the user to easily and painlessly convert between *all* of these
spaces easily, which is often in tension with Scarlet's commitment to transparency and
consistency. When in doubt, Scarlet prioritizes the *explicit* over the *implicit*: giving users
more information rather than less, and avoiding silent errors at all costs.

One of the places that this philosophy is most apparent is in Scarlet's handling of conversion
between color spaces. Some color spaces, particularly those that seek to model human perception
(like `CIELAB` and `CIE 1931`), have a *gamut* (a set of representable colors) that exceeds the
limits of human perception: not all valid `CIELAB` coordinates are visible to humans, and every
color that humans can see can be represented in `CIELAB`.

Other spaces, particularly those seeking to model the limited range of color reproduction on
computer monitors and in printers, have a gamut that does not cover all of human
perception. Standard RGB (in Scarlet, `RGBColor`) covers only around 35% of visible colors. This
diagram displays the gamuts of various color spaces:

![gamut diagram](https://en.wikipedia.org/wiki/SRGB#/media/File:CIE1931xy_gamut_comparison.svg)

# The Problem With Primaries
Many of the most popular color spaces are *primary-based*: each axis represents the intensity of
some base color that is then mixed in some fashion with the other axes' colors to get the final
result. This seems like it would match human perception: we have three kinds of color receptors, and
each of them has their highest affinity for a different frequency of light. Additionally, mixing
visible colors always produces another visible color: mathematically, we say that the full gamut of
human vision is *convex*.

The problem is that, although our vision's gamut is convex, it isn't a triangle: there are no three
visible colors that together can mix to generate *every* visible color. Blues and greens are the
biggest issue: as you can see from the diagram, there aren't really any purples that don't decompose
nicely into red and blue. (The reason for this is because the middle-wavelength "green" receptors
overlap a lot with the other receptors: there's no green that doesn't in part stimulate *all* of the
receptors in your eye.) This means that the only primary-based systems that can represent every
color don't actually use visible primaries! (Other color spaces solve this issue by using invisible
primaries or using an entirely different representation of color vision, such as `CIELAB` or
`CIELCH`.

# What This Means For Scarlet
The takeaway is that color spaces are presented to the user as essentially different unit systems,
all interconvertible. The rationale for this abstraction is that Scarlet aims to allow users to work
solely within color spaces that can make this abstraction work with zero cost (for example,
converting solely between `RGB` and `HSV`) without needing to worry about color spaces for which
this doesn't hold. Scarlet additionally promises that any *implicit* conversion it does, without
explicit invocation, won't cause errors or incorrect results.

If you are going to explicitly convert between color spaces that have different gamuts, Scarlet
trusts that you know what you're doing. The standard `convert()` method that is used to change color
spaces can and will *clamp* outputs to the gamut of the color spaces that they're in. This means
that conversion can create unexpected problems!

```rust
// this color doesn't exist in sRGB! (that's probably a good thing, this can't really be represented)
let color1 = CIELABColor{l: 0.0, a: 100.0, b: 100.0};
// this color gets clamped within the sRGB gamut: it ends up being #690000
let color2: RGBColor = color1.convert();
let color3: CIELABColor = color2.convert();
println!("{} {} {}", color3.l, color3.a, color3.b);
// prints the following:
// 20.604385926623827 42.096030499561316 31.81745246025879
```

# How to Avoid Gamut Problems
Scarlet provides plenty of tools to help you anticipate and correct for problems in gamuts.

TODO: actually do this
