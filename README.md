# RC.Machine

RC.Machine is a client/server application to control Raspberry PI based robot

## Installation

### Server

Installation is done as a systemd service.

- First you have to generate an authorization token:

    ```console
    ~$ openssl rand -base64 64
    ```

- Create `rc.server.service` file into the `/etc/systemd/system/` directory with following content:

    ```bash
    #
    # /etc/systemd/system/rc.server.service
    #
    [Unit]
    Description=RC.Machine Server
    Requires=network-online.target

    [Service]
    WorkingDirectory=/opt/rc.machine/
    Type=simple
    Restart=always
    RemainAfterExit=yes
    User=root
    ExecStart=server
    Environment=RUST_LOG=debug
    Environment=RUST_BACKTRACE=full
    Environment=RC_TOKEN=<token>
    Environment=RC_PORT=20301

    [Install]
    WantedBy=default.target
    ```

    you also need to update the `RC_TOKEN` value with an authorization token created in a previous step.

- Systemd script is configured to run the binary from ``/opt/rc.machine/``. You should prepare that working directory on your Raspberry Pi server:

    ```console
    raspberrypi.local~$ sudo mkdir -p /opt/rc.machine
    raspberrypi.local~$ sudo chown %username% /opt/rc.machine
    ```

- Set up ARMv7 cross compilation toolchain:

    ```console
    brew install arm-linux-gnueabihf-binutils
    rustup target add armv7-unknown-linux-musleabihf
    ```

- Build and upload a service binary via SSH:

    ```console
    # setup SSH connection to your Raspberry PI machine
    echo "SSH_HOST=%raspberry.host%" >> .env
    echo "SSH_PORT=%raspberry.user%" >> .env
    echo "SSH_USER=%raspberry.port%" >> .env

    # cross-compile and upload the server binary
    make sync
    ```

- Enable a service:

    ```console
    sudo systemctl enable rc.server.service  # enables startup on boot
    sudo systemctl start rc.server.service
    ```

### Client

Update `connection.token` section in a `Settings.toml` file with your authorization token and run `make` to build and run the client.
