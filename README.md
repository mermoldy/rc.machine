# RC.Machine

RC.Machine is a client/server application to control Raspberry PI based robot

## Install

### Client

- TODO

### Server

- [As systemd service](linux-systemd/README.md)
- [Via docker](docker/README.md) (TODO)

## Build

### Build Client

```console
make run
```

### Build Server

- Set up ARMv7 cross compilation toolchain for the MacOS:

```console
brew install arm-linux-gnueabihf-binutils
rustup target add armv7-unknown-linux-musleabihf
```

- Build and upload service via SSH:

```console
# setup SSH connection to your Raspberry PI machine
echo "SSH_HOST=%raspberry.host%" >> .env
echo "SSH_PORT=%raspberry.user%" >> .env
echo "SSH_USER=%raspberry.port%" >> .env

# build and upload the server binary
make sync
```
