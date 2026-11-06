# Show the page on a Seeed EE02 panel

Until birdnet-go-frame's own Rust firmware is ready, use fugleramme's MicroPython firmware for the
EE02: it reads the same `/frame.e6` format.

## You need

- Seeed XIAO ePaper Display Board EE02 with the Seeed 13.3" Spectra 6 panel
- A server running and reachable from your Wi-Fi (see
  [Connect to your BirdNET-Go](connect-birdnet-go.md))

## Steps

1. Flash MicroPython's `SEEED_XIAO_ESP32S3` build to the board (it needs the board's octal PSRAM).
2. Copy `boot.py`, `main.py` and `config.py` from fugleramme's
   [`examples/ee02`](https://github.com/arnegiacomo/fugleramme/tree/main/examples/ee02) to the board.
3. In `config.py`, set:

   ```python
   WIFI_NETWORKS = (("your-network", "your-password"),)
   FRAME_HOST = "192.168.1.20"   # the machine running birdnet-go-frame
   FRAME_PORT = 8081             # the port you published
   ```

4. Reset the board. Its LED shows short-long-short once Wi-Fi is up and stays on during the
   refresh, which takes about 30 seconds.

The panel then sleeps for the `POLL_MINUTES` the server sends (never under the firmware's own
minimum, 5 minutes by default) and redraws only when the page changes.

## If the panel shows an error

| Message on the panel | Fix |
| --- | --- |
| "The frame is not serving this panel" | Check the URL and port; `/frame.e6` must answer 200 |
| "lays its page out for a … panel" | The server only packs for the 13.3" panel |
| "Cannot reach the frame" | Check `FRAME_HOST`, the port, and that the board and server share a network |
