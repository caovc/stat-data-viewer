#!/usr/bin/env python3
"""Generate navy grid icons without third-party image libraries."""

from __future__ import annotations

import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "src-tauri" / "icons"


def png(width: int, height: int) -> bytes:
    rows = bytearray()
    for y in range(height):
        rows.append(0)
        for x in range(width):
            edge = x < 2 or y < 2 or x >= width - 2 or y >= height - 2
            grid = x % max(width // 8, 4) == 0 or y % max(height // 8, 4) == 0
            if edge:
                r, g, b = 15, 38, 60
            elif grid:
                r, g, b = 31, 78, 121
            else:
                r, g, b = 232, 236, 240
            # mark a cell in the upper-left like a selected spreadsheet cell
            if width // 5 < x < width // 2 and height // 5 < y < height // 2:
                r, g, b = 31, 78, 121
            rows.extend((r, g, b, 255))

    def chunk(tag: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + tag + data + struct.pack(
            ">I", zlib.crc32(tag + data) & 0xFFFFFFFF
        )

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(bytes(rows), 9))
        + chunk(b"IEND", b"")
    )


def ico(png_bytes: bytes, size: int) -> bytes:
    # PNG-in-ICO (Vista+)
    header = struct.pack("<HHH", 0, 1, 1)
    entry = struct.pack(
        "<BBBBHHII",
        size if size < 256 else 0,
        size if size < 256 else 0,
        0,
        0,
        1,
        32,
        len(png_bytes),
        22,
    )
    return header + entry + png_bytes


def main() -> None:
    ROOT.mkdir(parents=True, exist_ok=True)
    p32 = png(32, 32)
    p128 = png(128, 128)
    p256 = png(256, 256)
    (ROOT / "32x32.png").write_bytes(p32)
    (ROOT / "128x128.png").write_bytes(p128)
    (ROOT / "128x128@2x.png").write_bytes(p256)
    (ROOT / "icon.ico").write_bytes(ico(p256, 256))
    # Minimal ICNS with a PNG 256 icon (ic08)
    data = p256
    icns = b"icns" + struct.pack(">I", 8 + 8 + len(data)) + b"ic08" + struct.pack(">I", 8 + len(data)) + data
    (ROOT / "icon.icns").write_bytes(icns)
    print(f"wrote icons in {ROOT}")


if __name__ == "__main__":
    main()
