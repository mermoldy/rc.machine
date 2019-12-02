# Cat.Hunter is a client/server application to control Raspberry PI based robot 🤖🐈


## Video Streaming

```
mkdir -p /opt/streameye
python3 -m pip install picamera
git clone https://github.com/ccrisan/streameye.git
cd streameye
make
sudo make install
```

```
#!/bin/bash
BASE_DIR="/opt/streameye"

python3 ${BASE_DIR}/extras/raspimjpeg.py \
  -w 1200 \
  -h 800 \
  -r 30 \
  -q 10 \
  --rotation=90 | \
  ${BASE_DIR}/streameye -d "\xff\xd9\xff\xd8" -p 8081


```

```
#
# /etc/systemd/system/streameye.service
#
[Unit]
Description=StreamEye MJPEG Service

[Service]
ExecStart=/opt/cat.hunter/streameye.sh
Restart=on-failure
RestartSec=2

[Install]
WantedBy=multi-user.target
```
