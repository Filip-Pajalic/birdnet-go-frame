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

The image is `FROM scratch` and holds only the static server binary (about 1.5 MB, the font and
the bird-size table included) and `fake-birdnet`. It runs as an unprivileged user.

## Run it

```bash
docker run -d --name birdnet-go-frame -p 8081:8080 \
  -e BIRDNET_URL=http://192.168.1.10:8080 \
  -e UTC_OFFSET_MINUTES=60 \
  ghcr.io/filip-pajalic/birdnet-go-frame:edge
```

BirdNET-Go usually holds port 8080 on the host, so the example publishes the frame on 8081.

## On Unraid

1. **Docker** tab, **Add Container**.
2. Repository: `ghcr.io/filip-pajalic/birdnet-go-frame:edge` (or a release tag, or your own build).
3. Port: container `8080`, host `8081`.
4. Variables: `BIRDNET_URL` (BirdNET-Go's address as the container sees it, e.g. the Unraid IP and
   BirdNET-Go's port) and `UTC_OFFSET_MINUTES`. Add any others from
   [Configuration](../reference/configuration.md).
5. Apply, then open `http://<unraid-ip>:8081/`.

Give Unraid a fixed IP or a reliable hostname, so the panel always finds the server.

## Try it without BirdNET-Go

```bash
docker run --rm -p 8080:8080 -e BIRDNET_URL=mock birdnet-go-frame
```
