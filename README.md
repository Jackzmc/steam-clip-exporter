Steam Clip Exporter
===================

Exporting video files from Steam Game Recording doesn't work for me, so I make
clips and use this to get an mp4 file instead.

Installation
------------

```sh
$ cargo install --git=https://github.com/darkwater/steam-clip-exporter
```

Usage
-----

1. Run `steam-clip-exporter [clip name]*`
2. Select clip to export
3. `ffmpeg` is used to glue the chunks together in a fraction of a second, no re-encoding
4. Path of output file is printed to stdout

\* If clip name is not set, shows menu. Otherwise, you can directly provide clip name to export

Suggestion:

```sh
cp $(steam-clip-exporter) ./output.mp4
```

(Better UX may come later)
