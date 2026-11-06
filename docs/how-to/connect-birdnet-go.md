# Connect to your BirdNET-Go

1. Find BirdNET-Go's address on your network, e.g. `http://192.168.1.10:8080`. Opening it in a
   browser should show BirdNET-Go's dashboard.
2. Check the API answers:

   ```bash
   curl "http://192.168.1.10:8080/api/v2/detections/recent?limit=1"
   ```

   You should get a JSON list. A `401` means BirdNET-Go's private mode is on, which the server
   does not support yet; turn it off or allow your LAN in BirdNET-Go's security settings.
3. Start the server with the address and your time zone (minutes east of UTC):

   ```bash
   BIRDNET_URL=http://192.168.1.10:8080 UTC_OFFSET_MINUTES=60 birdnet-go-frame
   ```

4. Open `http://<server>:8080/` and check the species match BirdNET-Go's dashboard.

If BirdNET-Go is unreachable at start-up, the page says "Cannot reach BirdNET-Go" with the reason
in its footer. Once a real page has been drawn, it stays up through later outages.

See [Configuration](../reference/configuration.md) for every setting.
