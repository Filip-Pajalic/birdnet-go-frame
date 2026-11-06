# Connect to your BirdNET-Go

1. Find BirdNET-Go's address on your network, e.g. `http://192.168.1.10:8080`. Opening it in a
   browser should show BirdNET-Go's dashboard.
2. Check the API answers:

   ```bash
   curl "http://192.168.1.10:8080/api/v2/detections/recent?limit=1"
   ```

   You should get a JSON list. A `401` means BirdNET-Go's private mode is on: see
   [If BirdNET-Go is private](#if-birdnet-go-is-private).
3. Start the server with the address and your time zone (minutes east of UTC):

   ```bash
   BIRDNET_URL=http://192.168.1.10:8080 UTC_OFFSET_MINUTES=60 birdnet-go-frame
   ```

4. Open `http://<server>:8080/` and check the species match BirdNET-Go's dashboard.

If BirdNET-Go is unreachable at start-up, the page says "Cannot reach BirdNET-Go" with the reason
in its footer. Once a real page has been drawn, it stays up through later outages.

## If BirdNET-Go is private

With `security.privatemode` on, BirdNET-Go answers `401` until a client logs in. Either:

- **Let the server log in.** With BirdNET-Go's password login on (`security.basicauth`), give
  the server its password:

  ```bash
  BIRDNET_URL=http://192.168.1.10:8080 BIRDNET_PASSWORD='your password' birdnet-go-frame
  ```

  The server logs in the way BirdNET-Go's own page does, keeps the session, and logs in again
  when the session expires. `BIRDNET_USERNAME` defaults to `admin`; the password login takes any
  name. With a wrong password the server logs "BirdNET-Go refused the login: check
  BIRDNET_PASSWORD", and so does the page if none has been drawn yet.
- **Or let your LAN in without a login.** BirdNET-Go's subnet bypass
  (`security.allowsubnetbypass` with your LAN's subnet, e.g. `192.168.1.0/24`) admits requests
  from that subnet. On Unraid, the frame's container must then reach BirdNET-Go from an address
  in that subnet, e.g. through the host's IP.

Logging in with Google or GitHub (BirdNET-Go's social logins) is not supported: those need a
browser.

See [Configuration](../reference/configuration.md) for every setting.
