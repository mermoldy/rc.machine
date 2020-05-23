URL=raspberrypi.local
INSTALL_DIR=/opt/rc.machine
USER=mermoldy
SSH_PORT=22

SERVER_TARGET=armv7-unknown-linux-musleabihf

default: run

run:
	cargo fmt
	RUST_BACKTRACE=full RUST_LOG=client=debug cargo run

build:
	cargo fmt
	RUST_BACKTRACE=full RUST_LOG=client=debug cargo build --release

# Setup ARMv7 Toolchain for MacOS:
# - brew install arm-linux-gnueabihf-binutils
# - rustup target add armv7-unknown-linux-musleabihf
update_server:
	@echo "Build and upload rc.machine files to $(URL):$(INSTALL_DIR)..."
	cargo fmt
	cargo build --release --workspace=server --bin=server --target=$(SERVER_TARGET)
	# -Z config-profile
	# -Z config-profile
	rsync -e "ssh -p $(SSH_PORT)" Settings.toml "$(USER)@$(URL):$(INSTALL_DIR)"
	rsync -e "ssh -p $(SSH_PORT)" ./target/$(SERVER_TARGET)/release/server "$(USER)@$(URL):$(INSTALL_DIR)"
	@echo "Done"
	make restart

# Install the server
#
# Post setup:
# raspberrypi 🍓 ➜ ~ sudo cat /etc/sudoers.d/username
# %username ALL= NOPASSWD: /bin/systemctl start rc.server
# %username ALL= NOPASSWD: /bin/systemctl stop rc.server
# %username ALL= NOPASSWD: /bin/systemctl restart rc.server
install:
	@echo "Creating $(INSTALL_DIR) directory..."
	## ssh -t -p $(SSH_PORT) $(USER)@$(URL) sudo mkdir -p $(INSTALL_DIR)


restart:
	@echo "Restarting rc.server service..."
	ssh -t -p $(SSH_PORT) $(USER)@$(URL) "sudo systemctl restart rc.server"
	@echo "Done"

# Setup:
# raspberrypi 🍓 ➜ ~ sudo usermod -a -G systemd-journal username
tail:
	@echo "Starring rc.server server on $(URL)..."
	ssh -t -p $(SSH_PORT) $(USER)@$(URL) "tail -f /var/log/rc.server.log -n 120"
	@echo "Done"

clean:
	rm -rf target
