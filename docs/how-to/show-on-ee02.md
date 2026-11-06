# Show the page on a Seeed EE02 panel

Until birdnet-go-frame's own Rust firmware is ready, use fugleramme's MicroPython firmware for the
EE02 with a few lines changed: it was written for `/frame.e6`, and this server sends the same
pixels at `/panel`, compressed and without the `.e6` header.

## You need

- Seeed XIAO ePaper Display Board EE02 with the Seeed 13.3" Spectra 6 panel
- A server running and reachable from your Wi-Fi (see
  [Connect to your BirdNET-Go](connect-birdnet-go.md))
- MicroPython 1.21 or newer, which has the `deflate` module

## Steps

1. Flash MicroPython's `SEEED_XIAO_ESP32S3` build to the board (it needs the board's octal PSRAM).
2. Copy `boot.py`, `main.py` and `config.py` from fugleramme's
   [`examples/ee02`](https://github.com/arnegiacomo/fugleramme/tree/main/examples/ee02).
3. In `config.py`, set:

   ```python
   WIFI_NETWORKS = (("your-network", "your-password"),)
   FRAME_HOST = "192.168.1.20"   # the machine running birdnet-go-frame
   FRAME_PORT = 8081             # the port you published
   ```

4. In `main.py`, add `import deflate` at the top, and in `_get` add the line
   `request += "Accept-Encoding: deflate\r\n"` next to the one that adds `If-None-Match`.
5. In `fetch`, ask for `/panel?panel=13.3` instead of `/frame.e6`, and replace everything from
   reading the header to the CRC check with:

   ```python
           pixels = bytearray(WIDTH * HEIGHT // 2)
           read_exactly(deflate.DeflateIO(sock, deflate.ZLIB), pixels)
           # The ETag is the CRC-32 of the inflated bytes, in hex.
           if binascii.crc32(pixels) != int(new_etag.strip('"').removesuffix("-z"), 16):
               raise Problem("The page arrived damaged.")
           return (pixels, new_etag), None
   ```

   The server's poll time now arrives as `Cache-Control: max-age`, which this firmware does not
   read, so it sleeps for its own `MIN_SLEEP_MINUTES` (5 by default); raise that in `config.py`
   to poll less often.

6. Copy the edited `main.py` to the board and reset it. Its LED shows short-long-short once Wi-Fi
   is up and stays on during the refresh, which takes about 30 seconds.

The panel redraws only when the page changes. These steps have not yet been tried on hardware;
please open an issue with what you find.

## If the panel shows an error

| Message on the panel | Fix |
| --- | --- |
| "The frame is not serving this panel" | Check the URL and port; `/panel` must answer 200 |
| "The page arrived damaged" | A short or broken download; it retries on the next wake |
| "Cannot reach the frame" | Check `FRAME_HOST`, the port, and that the board and server share a network |
