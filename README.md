

## Resources

- MLT XML: https://www.mltframework.org/docs/mltxml/

## Configuration

The checked-in [cut-bot.conf](cut-bot.conf) file is copied by the build process into the executable directory, for example `target/debug/cut-bot.conf`.

Update `ffmpeg_executable` there to point to your local `ffmpeg.exe`.

You can also set `magick_executable` if ImageMagick is not available on `PATH`.

`drawio_executable` defaults to `C:\Program Files\draw.io\draw.io.exe` in the shipped config.

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

`cut-bot transparent <pattern>` applies the equivalent of:

```powershell
magick <file> -transparent "#1e1e1e" <basename>.transparent.png
```

Example:

```powershell
cut-bot transparent "code*.png"
```

`cut-bot drawio <input.drawio>` exports each page of the draw.io document as a transparent PNG using `drawio_executable` from the config.

Example:

```powershell
cut-bot drawio "diagram.drawio"
```
