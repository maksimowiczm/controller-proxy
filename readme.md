```
Usage: controller-proxy [OPTIONS] <COMMAND>

Commands:
  tcp    Use TCP socket
  usb    Use XBOX controller USB event file
  file   Use file
  stdin  Use stdin
  auto   Auto mode will atempt to use either USB controller or a TCP socket
  help   Print this message or the help of the given subcommand(s)

Options:
      --file <FILE>            Write output to file
      --serial <SERIAL>        Write output to serial
      --baud-rate <BAUD_RATE>  Serial baud rate [default: 9600]
      --stdout                 Write to stdout
  -h, --help                   Print help
  -V, --version                Print version

```

Example usage

Use RUST_LOG to display logger messages.

```bash
export RUST_LOG=debug
export RUST_LOG=info
```

Start with cargo
```bash
export RUST_LOG=info && cargo run --release -- auto
```

Start with docker

TCP -> Serial

```bash
docker run \
--device /dev/ttyACM0 \
-e RUST_LOG=info \
-p 8080:8080 \
maksimowiczm/controller-proxy tcp 0.0.0.0 8080 \
--serial /dev/ttyACM0
```

Auto mode with controller /dev/input/event0 and serial /dev/ttyACM0

```bash
docker run \
--device /dev/ttyACM0 \
--device /dev/input/event0 \
-e RUST_LOG=info \
-p 8080:8080 \
maksimowiczm/controller-proxy auto \
--loop \
--serial /dev/ttyACM0
```
