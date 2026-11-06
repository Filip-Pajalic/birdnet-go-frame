# Show frames in Home Assistant

Each EE02 frame sends its battery voltage and firmware version when it wakes. The server keeps
the last of each per frame, and can pass them to Home Assistant two ways.

## With MQTT (the frames appear by themselves)

You need an MQTT broker that Home Assistant uses, such as the Mosquitto add-on, with the MQTT
integration set up.

1. Point the server at the broker:

   ```text
   MQTT_URL=mqtt://frames:a-password@192.168.1.5:1883
   ```

   Leave out `frames:a-password@` if the broker takes anonymous clients. If you changed Home
   Assistant's discovery prefix, set `MQTT_DISCOVERY_PREFIX` too.

2. Restart the server. It logs `announcing frames to Home Assistant at 192.168.1.5:1883`.

3. Wait for a frame to wake. A device called **Bird frame hall** (after the frame's name)
   appears under the MQTT integration, with these sensors:

   | Sensor | Shows |
   | --- | --- |
   | Battery | Charge in percent, read roughly off the voltage |
   | Battery voltage | Millivolts, as the frame measured them (diagnostic) |
   | Last wake | When it last asked for its page |
   | Page | The view it asked for, empty for its default |
   | Firmware | Its firmware's version (diagnostic) |

The server checks for new wakes every 15 seconds and publishes them retained, so Home Assistant
has them after a restart too. A broker it cannot reach is tried again at the next check.

A useful automation: notify when Battery is below 15 %, or when Last wake is more than an hour
old (the frame has stopped waking).

## Without MQTT (REST)

The same facts are at `http://server:8080/api/frames/<name>`. A REST sensor in
`configuration.yaml`:

```yaml
rest:
  - resource: http://192.168.1.10:8080/api/frames/hall
    scan_interval: 300
    sensor:
      - name: Hall frame battery
        value_template: "{{ value_json.battery_percent }}"
        unit_of_measurement: "%"
        device_class: battery
      - name: Hall frame last wake
        value_template: "{{ as_datetime(value_json.last_wake) }}"
        device_class: timestamp
```

The answer is `404` until the frame has woken since the server started. See
[HTTP endpoints](../reference/http-endpoints.md#frames-wakes) for every field.
