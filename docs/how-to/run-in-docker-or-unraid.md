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

1. **Docker** tab, **Add Container**.
2. Repository: `ghcr.io/filip-pajalic/birdnet-go-frame:edge` (or a release tag, or your own build).
3. Port: container `8080`, host `8081`.
4. Variables: `BIRDNET_URL` (BirdNET-Go's address as the container sees it, e.g. the Unraid IP and
   BirdNET-Go's port), `UTC_OFFSET_MINUTES`, `LANGUAGE` and `ART_DIR=/art/classic`. Add a path
   mapping from `/mnt/user/appdata/birdnet-go-frame/art` to `/art` (read only). Add any others from
   [Configuration](../reference/configuration.md).
5. Apply, then open `http://<unraid-ip>:8081/`.

Give Unraid a fixed IP or a reliable hostname, so the panel always finds the server.

## Try it without BirdNET-Go

```bash
docker run --rm -p 8080:8080 -e BIRDNET_URL=mock birdnet-go-frame
```
