# tinyproxy ![Release](https://github.com/skarrok/tinyproxy/actions/workflows/release.yml/badge.svg)

A tiny and simple proxy with http, socks5 and tcp support.

## Quickstart

Download latest release from github releases page and run it with the following command:

```sh
# http mode
tinyproxy http --listen-address 0.0.0.0:8000
# socks5 mode
tinyproxy socks5 --listen-address 0.0.0.0:8000
# tcp mode
tinyproxy tcp --listen-address 0.0.0.0:8000 --remote-address 127.0.0.1:443
```

## Configuration

You can pass configuration parameters as command line arguments or environment variables.

```txt
A tiny and simple proxy with http, socks5 and tcp support

Usage: tinyproxy [OPTIONS] <PROXY_MODE>

Arguments:
  <PROXY_MODE>
          [env: PROXY_MODE=]
          [possible values: http, socks5, tcp]

Options:
      --log-level <LOG_LEVEL>
          Verbosity of logging

          [env: LOG_LEVEL=]
          [default: debug]
          [possible values: off, trace, debug, info, warn, error]

      --log-format <LOG_FORMAT>
          Format of logs

          [env: LOG_FORMAT=]
          [default: console]

          Possible values:
          - console: Pretty logs for debugging
          - json:    JSON logs

  -l, --listen-address <LISTEN_ADDRESS>
          Listen address

          [env: LISTEN_ADDRESS=]
          [default: 127.0.0.1:8000]

  -r, --remote-address <REMOTE_ADDRESS>
          Remote address for TCP mode

          [env: REMOTE_ADDRESS=]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Building

It is as simple as cloning this repository and running

```bash
cargo build --release
```
