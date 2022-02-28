ffmpeg -i %d_ok.png -vf palettegen palette.png && ffmpeg -v warning -i %d_ok.png -i palette.png  -lavfi "paletteuse,setpts=2*PTS" -y out.gif
