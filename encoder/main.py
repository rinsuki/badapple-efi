#!/usr/bin/env python3
import subprocess
import sys
import os
import json
import struct
from PIL import Image

# same as src/main.rs const
WIDTH = 512
HEIGHT = 384
FPS = int(30)

if os.environ.get("SKIP_FFMPEG_CONVERT") != "1":
    subprocess.run(["ffmpeg", "-i", sys.argv[1], "-s", "{}x{}".format(WIDTH, HEIGHT), "-pix_fmt", "gray", "-r", str(FPS), "png/%04d.png"])
    subprocess.run(["ffmpeg", "-i", sys.argv[1], "-codec", "copy", "bin/music.mp4"])

frames = 0
with open("bin/data.bin", "wb") as fb:
    with open("bin/seek.bin", "wb") as fs:
        for png in sorted(os.listdir("png")):
            frames += 1
            print(png)
            if not png.endswith(".png"):
                continue
            with Image.open("png/" + png) as img:
                b = bytes(img.getdata(0))
                l = len(b)
                i = 0
                while i < l:
                    if i == l-1:
                        fb.write(bytes([
                            0b1000_0000,
                            b[i]
                        ]))
                        i += 1
                    elif b[i] == b[i+1]:
                        # same
                        cnt = 0
                        current = b[i]
                        while i < l and b[i] == current:
                            cnt += 1
                            i += 1
                        cnt -= 1
                        if cnt <= 0b11111:
                            fb.write(bytes([cnt, current]))
                        elif cnt <= 0b11111111_11111:
                            fb.write(bytes([
                                0b001_00000 | (cnt & 0b11111),
                                cnt >> 5,
                                current,
                            ]))
                        elif cnt <= 0b11111111_11111111_11111:
                            fb.write(bytes([
                                0b010_00000 | (cnt & 0b11111),
                                (cnt >> 5) & 0b11111111,
                                (cnt >> 13) & 0b11111111,
                                current,
                            ]))
                        else:
                            raise "big cnt"
                    else:
                        # not same
                        cnt = 0
                        oi = i
                        while i < l:
                            if i >= l - 2:
                                pass
                            elif b[i] == b[i+1] and b[i+1] == b[i+2]:
                                break
                            cnt += 1
                            i += 1
                        cnt -= 1
                        if cnt <= 0b11111:
                            fb.write(bytes([0b100_00000 | cnt]))
                        elif cnt <= 0b11111111_11111:
                            fb.write(bytes([
                                0b101_00000 | (cnt & 0b11111),
                                cnt >> 5
                            ]))
                        elif cnt <= 0b11111111_11111111_11111:
                            fb.write(bytes([
                                0b110_00000 | (cnt & 0b11111),
                                (cnt >> 5) & 0b11111111,
                                (cnt >> 13) & 0b11111111,
                            ]))
                        else:
                            raise "big cnt"
                        if len(b[oi:i]) != (cnt+1) or cnt+1 != i-oi:
                            raise "?"
                        fb.write(b[oi:i])
                fs.write(struct.pack("<I", fb.tell()))
