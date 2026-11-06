# Run the server in Docker or on Unraid

## Build the image

```bash
docker build -f deploy/Dockerfile -t birdnet-go-frame .
```

The image is `FROM scratch` and holds only the static server binary (about 1.8 MB, most of it the
embedded font) and `fake-birdnet`.

## Run it

```bash
docker run -d --name birdnet-go-frame -p 8081:8080 \
  -e BIRDNET_URL=http://192.168.1.10:8080 \
  -e UTC_OFFSET_MINUTES=60 \
  birdnet-go-frame
```

BirdNET-Go usually holds port 8080 on the host, so the example publishes the frame on 8081.

## On Unraid

1. **Docker** tab, **Add Container**.
2. Repository: the image you built or pushed (e.g. `ghcr.io/<you>/birdnet-go-frame`).
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
