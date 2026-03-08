

## Resources

- MLT XML: https://www.mltframework.org/docs/mltxml/

## Configuration

The checked-in [cut-bot.conf](cut-bot.conf) file is copied by the build process into the executable directory, for example `target/debug/cut-bot.conf`.

Update `ffmpeg_executable` there to point to your local `ffmpeg.exe`.

You can also set `magick_executable` if ImageMagick is not available on `PATH`.

## Commands

`cut-bot silence <input> <output.mlt>` creates a ShotCut project with silent parts marked.

`cut-bot border <pattern>` applies the equivalent of:

```powershell
magick <file> -bordercolor black -border 8 <basename>.border.png
```

Example:

```powershell
cut-bot border "thumbnail-*.png"
```
