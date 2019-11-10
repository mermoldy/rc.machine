#/usr/bin/bash

URL=raspberrypi.local
INSTALL_DIR=/opt/cat.hunter
USER=mermoldy
SSH_PORT=22

echo "Starring cat.hunter server on $URL..."
ssh -t -p $SSH_PORT $USER@$URL "cd $INSTALL_DIR ; tail -f /var/log/cat.hunter.log"
echo "Done"
