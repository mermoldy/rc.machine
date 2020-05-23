# #
# Development tasks
# #

# Remote machine configuration
# You can override this settings via .env file
SSH_HOST=raspberrypi.local
SSH_USER=raspberrypi
SSH_PORT=22

include .env
export

INSTALL_DIR=/opt/rc.machine
SERVER_TARGET=armv7-unknown-linux-musleabihf

# #
# Client tasks
# #

#
#  Run GUI client on the local machine
#
default: run
run:
	cargo fmt
	RUST_BACKTRACE=full RUST_LOG=client=debug cargo run --release

# #
# Server tasks (via SSH)
# #

#
#  Build and upload armv7 binary to the remote Raspberry PI machine
#
sync:
	@echo "Build and upload rc.machine files to $(SSH_HOST):$(INSTALL_DIR)..."
	cargo build --release --workspace=server --bin=server --target=$(SERVER_TARGET) 
	rsync -e "ssh -p $(SSH_PORT)" Settings.toml "$(SSH_USER)@$(SSH_HOST):$(INSTALL_DIR)"
	rsync -e "ssh -p $(SSH_PORT)" ./target/$(SERVER_TARGET)/release/server "$(SSH_USER)@$(SSH_HOST):$(INSTALL_DIR)"
	ssh -t -p $(SSH_PORT) $(SSH_USER)@$(SSH_HOST) "sudo systemctl restart rc.server"
	@echo "Done"

#
#  Listen logs
#
# To setup:
# > sudo usermod -a -G systemd-journal %username%
tail:
	@echo "Starring rc.server server on $(SSH_HOST)..."
	ssh -t -p $(SSH_PORT) $(SSH_USER)@$(SSH_HOST) "tail -f /var/log/rc.server.log -n 120"
	@echo "Done"

# #
# Common tasks
# #

#
#  Remove cache
#
clean:
	rm -rf target
