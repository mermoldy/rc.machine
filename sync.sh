#/usr/bin/bash

URL=raspberrypi.local
INSTALL_DIR=/opt/cat.hunter
USER=mermoldy
SSH_PORT=22

echo "Syncing cat.hunter files to $URL:$INSTALL_DIR..."
rsync \
    \
    --exclude=.git \
    --exclude=.mypy_cache \
    --exclude=.gitignore \
    --exclude=*/__pycache__ \
    --exclude=data \
    \
    -e "ssh -p $SSH_PORT" \
    -r . "$USER@$URL:$INSTALL_DIR"
echo "Done"

echo "Restarting cat.hunter service..."
ssh -t -p $SSH_PORT $USER@$URL "cd $INSTALL_DIR ; sudo systemctl restart cat.hunter"
echo "Done"
