# Give each frame its own page

One server can feed several frames, each with its own layout, panel and page: a big collage in
the hall, the day's story on a small panel on the desk.

1. Name the frames and say what each shows in `FRAMES`:

   ```text
   FRAMES=hall:layout=collage;desk:layout=list,panel=7.3,view=story
   ```

   Anything a frame leaves out comes from the server's own settings (`LAYOUT`, `PANEL`,
   `ROTATION`, `VIEW`, `MAX_BIRDS`, `NAMES`, `HEADING`). The settings are listed in
   [Configuration](../reference/configuration.md#frames).

2. Give each frame its name. On the EE02, it is the frame name on its
   [setup page](set-up-ee02-over-wifi.md). Anything else asks with `frame=`, e.g.
   `http://server:8080/panel?frame=desk`.

3. Check each frame's page in a browser:

   ```text
   http://server:8080/preview.png?frame=hall
   http://server:8080/preview.png?frame=desk
   http://server:8080/preview.png?frame=desk&view=season
   ```

A frame can still be asked for another page with `view=` (the EE02's button does this). A
frame that gives no name, or one not in `FRAMES`, gets the server's own settings, so adding a
frame never breaks the ones you have.

To see the pages without a microphone, run the server with `BIRDNET_URL=mock`, and
`MOCK_DATE=2026-05-01` to see spring's arrivals. See [Pages and views](../explanation/pages-and-views.md)
for what each page shows.
