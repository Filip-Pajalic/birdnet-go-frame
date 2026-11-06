# Run the server in Docker or on Unraid

## Get the image

Published images for amd64 and arm64 are on GitHub Container Registry:

| Tag | Follows |
| --- | --- |
| `ghcr.io/filip-pajalic/birdnet-go-frame:edge` | the `main` branch |
| `ghcr.io/filip-pajalic/birdnet-go-frame:0.2.0`, `:0.2` | releases |

Or build it yourself:

```bash
docker build -f deploy/Dockerfile -t birdnet-go-frame .
```

The image is `FROM scratch` and holds only the static server binary (about 1.7 MB, the font,
the bird-size table and the translations included) and `fake-birdnet`. It runs as an unprivileged user.

## Run it

```bash
docker run -d --name birdnet-go-frame -p 8081:8080 \
  -e BIRDNET_URL=http://192.168.1.10:8080 \
  -e UTC_OFFSET_MINUTES=60 \
  -e LANGUAGE=sv \
  -v /mnt/user/appdata/birdnet-go-frame/art:/art:ro \
  -e ART_DIR=/art/classic \
  ghcr.io/filip-pajalic/birdnet-go-frame:edge
```

The art folder holds fugleramme's plates, which are not in the image (they are about 250 MB and
CC BY-SA). Fill it once from any machine with git, e.g. the Unraid terminal:

```bash
scripts/fetch-art.sh classic /mnt/user/appdata/birdnet-go-frame/art
```

Leave out the `-v` and `ART_DIR` lines to draw generated birds instead.

BirdNET-Go usually holds port 8080 on the host, so the example publishes the frame on 8081.

## On Unraid

### Add the container from the template

1. Open a terminal on Unraid (the **>_** icon) and fetch the template:

   ```bash
   wget -O /boot/config/plugins/dockerMan/templates-user/my-birdnet-go-frame.xml \
     https://raw.githubusercontent.com/Filip-Pajalic/birdnet-go-frame/main/deploy/unraid/birdnet-go-frame.xml
   ```

2. **Docker** tab, **Add Container**, and pick **birdnet-go-frame** from the **Template** list.
3. Set **BirdNET-Go URL** to BirdNET-Go's address as the container sees it, e.g.
   `http://192.168.1.10:8080` (the Unraid IP and BirdNET-Go's port), and **Time zone offset**
   to your garden's (60 for Sweden in winter). Leave **Art style** empty for now.
4. **Apply**, then open `http://<unraid-ip>:8081/`. You should see generated birds for the
   species your station has heard today.

The template is in the repository at
[`deploy/unraid/birdnet-go-frame.xml`](../../deploy/unraid/birdnet-go-frame.xml). **Show more
settings** reveals the panel, rotation, poll time and layout.

### Add fugleramme's plates

Unraid has no git, so fetch the plates with a throwaway git container, from the same terminal:

```bash
wget -qO- https://raw.githubusercontent.com/Filip-Pajalic/birdnet-go-frame/main/scripts/fetch-art.sh |
  docker run --rm -i -v /mnt/user/appdata/birdnet-go-frame/art:/art --entrypoint sh alpine/git -s classic /art
```

It downloads about 250 MB and ends with `1355 plates in /art/classic`. Then edit the container,
set **Art style** to `/art/classic`, and apply. The page now uses the plates.

Give Unraid a fixed IP or a reliable hostname, so the panel always finds the server.

## Try it without BirdNET-Go

```bash
docker run --rm -p 8080:8080 -e BIRDNET_URL=mock birdnet-go-frame
```
