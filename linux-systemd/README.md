# Systemd service for RC.Machine

Systemd script for RC.Machine server.

## Installation

- Systemd script is configured to run the binary from /opt/rc.machine/.
- Download the binary. Find the relevant links for the binary at https://github.com/mermoldy/rc.machine/packages.

## Systemctl

Download `rc.server.service` in `/etc/systemd/system/`

```console
cd /etc/systemd/system/
sudo curl -O https://raw.githubusercontent.com/mermoldy/rc.machine/master/linux-systemd/rc.server.service
```

### Enable startup on boot

```console
systemctl enable rc.server.service
```
