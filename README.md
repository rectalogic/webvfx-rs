# WebVfx

Replacement for the defunct [Qt based WebVfx](https://github.com/rectalogic/webvfx).
Implemented as [frei0r](https://frei0r.dyne.org) plugins using the
[Blitz](https://blitz.is) web engine.

```
FREI0R_PATH=target/debug ffmpeg -t 5s -i https://assets.mixkit.co/videos/1479/1479-720.mp4 -vf 'frei0r=webvfx_filter:filter_params=effects/filter/banner.html|effects/filter/banner.json' -pix_fmt yuv420p -y output-ffmpeg.mp4
```
