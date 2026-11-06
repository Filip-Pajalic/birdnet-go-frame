# Set up and update the EE02 over Wi-Fi

Once [the firmware is on the board](flash-ee02-thin.md), it never needs the USB cable again: its
settings are entered from a phone on the frame's own Wi-Fi network, and new firmware comes from
the server. This guide covers both, and the frame's buttons.

> None of this has run on an EE02 yet; it builds, and its logic is tested on a laptop. Please open
> an issue with the serial log if something here does not happen as described.

## Enter or change the settings

The frame opens its setup hotspot by itself the first time, when it has no settings. To change
them later:

1. Hold the **refresh** button (the first of the three, KEY1).
2. While holding it, press and release **RESET**.
3. Keep holding refresh for 3 more seconds, then let go.

Then, on a phone or laptop:

1. Join the open Wi-Fi network called `BirdFrame-` followed by four letters and digits.
2. A "sign in to network" page should open by itself. If it does not, open
   `http://192.168.4.1/` in a browser.
3. Fill in the form:
   - **Wi-Fi name** and **password** of your home network (2.4 GHz). The names the frame saw
     nearby are offered as you type. When you only change something else, leave the password
     empty: the saved one is kept.
   - **Frame server**: the address and port your server is published on, e.g.
     `192.168.1.20:8081`. It has to be a fixed IPv4 address, not a name.
   - **Frame name** (optional): lower-case letters, digits and `-`. The server can give each name
     its own layout, e.g. a collage for `hall` and a list for `desk`.
   - **Rotation**: leave it on "as the server says" unless this frame hangs differently from the
     others.
4. Press **Save and restart**. The hotspot closes, the frame joins your network, and the first
   page appears within a minute or two.

If the form finds something wrong, it says what above the fields and keeps the hotspot open. The
hotspot closes by itself after 10 minutes; a frame that was already set up then carries on with its
old settings.

The hotspot is open (no password) for as long as it is up, so anyone in range could join it and
enter settings in those 10 minutes. It only opens when the frame has no settings or someone
holds the button, and the page never shows the saved password.

## Use the buttons

| Button | Does |
| --- | --- |
| Refresh | Wakes the frame and redraws the page now, even if it has not changed |
| Previous / Next | Wake the frame and step through the pages: today, this week, the rarest bird, the day's story, the season |

The frame remembers the page you picked until a button changes it. A button press also clears
any back-off after failures, so it is a quick way to see whether a fixed server is reachable.
Pages other than "today" need a server that knows them; one that does not shows its default page.

## Update the firmware over Wi-Fi

The server offers firmware from a folder, and each frame installs it at its next wake.

1. Build the new firmware and turn it into an app image (`espflash` comes with
   [the flashing guide](flash-ee02-thin.md)):

   ```bash
   cd apps/ee02-thin
   cargo build --release
   espflash save-image --chip esp32s3 \
     target/xtensa-esp32s3-none-elf/release/ee02-thin ee02-thin.bin
   ```

   The version the frame sees is the `version` in `apps/ee02-thin/Cargo.toml`; raise it for every
   release.

2. Put `ee02-thin.bin` in the server's firmware folder (`FIRMWARE_DIR`; see
   [Configuration](../reference/configuration.md)), with a file `ee02-thin.version` next to it
   holding the same version, e.g. `0.2.1`.

3. Wait for the next wake, or press refresh. The log shows:

   ```text
   ee02: the server offers firmware 0.2.1; downloading
   ee02: firmware 0.2.1 installed; restarting into it
   ee02: firmware 0.2.1
   ```

The frame checks the image before it writes anything: the right chip, every segment, the
checksum, and that its built-in name is `ee02-thin` and its version is the one in
`ee02-thin.version`. It writes the image to its spare slot and keeps the old firmware in the other,
so a bad download or a power cut costs nothing. It skips updates while the battery is under
3.6 V, and gives up on a version after three failed tries until a different one is offered.

To go back to an older firmware, offer the older version: any version other than the running one
is installed.
