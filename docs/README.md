# birdnet-go-frame documentation

The docs follow [Diátaxis](https://diataxis.fr/): four kinds of page for four different needs.

| You want to... | Read |
| --- | --- |
| **Learn** by doing, start to finish | [Tutorials](#tutorials) |
| **Get a specific job done** | [How-to guides](#how-to-guides) |
| **Look something up** | [Reference](#reference) |
| **Understand** how and why it works | [Explanation](#explanation) |

## Tutorials

- [Your first page, with the simulated BirdNET-Go](tutorials/first-page-with-the-mock.md)

## How-to guides

- [Connect to your BirdNET-Go](how-to/connect-birdnet-go.md)
- [Run the server in Docker or on Unraid](how-to/run-in-docker-or-unraid.md)
- [Build and flash the EE02 firmware](how-to/flash-ee02-thin.md)
- [Set up and update the EE02 over Wi-Fi, and use its buttons](how-to/set-up-ee02-over-wifi.md)
- [Show the page on an EE02 with fugleramme's MicroPython firmware](how-to/show-on-ee02.md)
- [Give each frame its own page](how-to/give-each-frame-its-own-page.md)
- [Show frames in Home Assistant](how-to/show-frames-in-home-assistant.md)
- [Develop and test without a microphone](how-to/develop-and-test.md)

## Reference

- [Configuration](reference/configuration.md)
- [HTTP endpoints](reference/http-endpoints.md)
- [The panel format](reference/panel-format.md)
- [Crates and apps](reference/crates.md)
- [The EE02 firmware: settings, setup hotspot, buttons, battery, updates, wake, memory, flash, pins](reference/firmware.md)
- [Supported hardware: panels, ESP32 models and combinations](reference/supported-hardware.md)

## Explanation

- [Architecture: how the pieces fit](explanation/architecture.md)
- [Pages and views: today, week, rarest, story, season, and the new-bird alert](explanation/pages-and-views.md)
- [How the image is made](explanation/image-pipeline.md)
- [Power, polling and refresh](explanation/power-and-refresh.md)
- [Design decisions](explanation/design-decisions.md)
- [Dependencies: why there are so few](explanation/dependencies.md)
