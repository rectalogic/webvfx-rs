Requires [GraphicsMagick](http://www.graphicsmagick.org/index.html)
```sh-session
$ brew install graphicsmagick ghostscript
```

`a-320x240.png`
```sh-session
$ gm convert \
  -size 300x220 xc:red \
  -bordercolor blue -border 10x10 \
  -font Helvetica-Bold.ttf \
  -fill white \
  -gravity center \
  -pointsize 200 \
  -draw "text 0,10 'A'" \
  -depth 8 \
  -type TrueColorMatte \
  a-320x240.png
```

`b-320x240.png`
```sh-session
$ gm convert \
  -size 300x220 xc:orange \
  -bordercolor green -border 10x10 \
  -font Helvetica-Bold.ttf \
  -fill white \
  -gravity center \
  -pointsize 200 \
  -draw "text 0,10 'B'" \
  -depth 8 \
  -type TrueColorMatte \
  b-320x240.png
```

`c-320x240.png`
```sh-session
$ gm convert \
  -size 300x220 xc:brown \
  -bordercolor yellow -border 10x10 \
  -font Helvetica-Bold.ttf \
  -fill white \
  -gravity center \
  -pointsize 200 \
  -draw "text 0,10 'C'" \
  -depth 8 \
  -type TrueColorMatte \
  c-320x240.png
```
