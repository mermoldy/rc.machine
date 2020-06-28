# Systemd service for RC.Machine

Systemd script for RC.Machine server.

## Installation

- Systemd script is configured to run the binary from ``/opt/rc.machine/``.
You should prepare the working directory:

```console
sudo mkdir -p /opt/rc.machine
sudo chown %username% /opt/rc.machine
```

- Download the server binary into ``/opt/rc.machine``. Find the relevant links for the binary at <https://github.com/mermoldy/rc.machine/packages> (TODO).

- Download `rc.server.service` into `/etc/systemd/system/`:

```console
cd /etc/systemd/system/
sudo curl -O https://raw.githubusercontent.com/mermoldy/rc.machine/master/linux-systemd/rc.server.service
```

- Start the service:

```console
sudo systemctl enable rc.server.service  # enables startup on boot
sudo systemctl start rc.server.service
```

```
Environment=RC_TOKEN=<token>
Environment=RC_PORT=20301
```