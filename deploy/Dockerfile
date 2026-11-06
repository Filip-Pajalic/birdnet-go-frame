# Static musl build, shipped in an empty image: the binary, its embedded font, nothing else.
#   docker build -f deploy/Dockerfile -t birdnet-go-frame .
#   docker run -p 8080:8080 -e BIRDNET_URL=mock birdnet-go-frame
FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY . .
# Use the image's toolchain rather than downloading the one rust-toolchain.toml names.
RUN rm -f rust-toolchain.toml && cargo build --release --locked -p server -p fake-birdnet

FROM scratch
COPY --from=build /src/target/release/birdnet-go-frame /birdnet-go-frame
COPY --from=build /src/target/release/fake-birdnet /fake-birdnet
ENV LISTEN=0.0.0.0:8080
USER 65534:65534
EXPOSE 8080
ENTRYPOINT ["/birdnet-go-frame"]
