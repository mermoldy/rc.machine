# Systemd service for RC.Machine

Systemd script for RC.Machine server.

## Installation

- Systemd script is configured to run the binary from /opt/rc.machine/.
- Download the binary. Find the relevant links for the binary at https://github.com/mermoldy/rc.machine/packages.

## Systemctl

Download `rc.server.service` and `rc.client.service`  in `/etc/systemd/system/`

```console
cd /etc/systemd/system/
curl -O https://raw.githubusercontent.com/mermoldy/rc.machine/master/linux-systemd/rc.server.service
curl -O https://raw.githubusercontent.com/mermoldy/rc.machine/master/linux-systemd/rc.client.service
```

```ini
[Service]
WorkingDirectory=/opt/rc.machine/
```

### Enable startup on boot

```console
systemctl enable rc.server.service
systemctl enable rc.client.service
```

### Disable RC.Machine service(s)

```console
systemctl disable rc.server.service
systemctl disable rc.client.service
```

## Note

- Replace ``User=rc-user`` and ``Group=rc-user`` in *.service files with your local setup.
