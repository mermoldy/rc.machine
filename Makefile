URL=raspberrypi.local
INSTALL_DIR=/opt/cat.hunter
USER=mermoldy
SSH_PORT=22

default: run

run:
	cargo fmt
	RUST_BACKTRACE=full RUST_LOG=debug cargo run

build:
	cargo fmt
	RUST_BACKTRACE=full RUST_LOG=debug cargo build

# Setup ARMv7 Toolchain for MacOS:
#  - brew install arm-linux-gnueabihf-binutils
#  - rustup target add armv7-unknown-linux-musleabihf
sync:
	@echo "Syncing cat.hunter files to $(URL):$(INSTALL_DIR)..."
	cargo build --workspace=server --bin=server --target=armv7-unknown-linux-musleabihf
	rsync -e "ssh -p $(SSH_PORT)" Settings.toml "$(USER)@$(URL):$(INSTALL_DIR)"
	rsync -e "ssh -p $(SSH_PORT)" ./target/armv7-unknown-linux-musleabihf/debug/server "$(USER)@$(URL):$(INSTALL_DIR)"
	@echo "Done"
	make restart

# Setup:
# raspberrypi 🍓 ➜ ~ sudo cat /etc/sudoers.d/mermoldy
# %mermoldy ALL= NOPASSWD: /bin/systemctl start cat.hunter
# %mermoldy ALL= NOPASSWD: /bin/systemctl stop cat.hunter
# %mermoldy ALL= NOPASSWD: /bin/systemctl restart cat.hunter
restart:
	@echo "Restarting cat.hunter service..."
	ssh -t -p $(SSH_PORT) $(USER)@$(URL) "sudo systemctl restart cat.hunter"
	@echo "Done"

# Setup:
# raspberrypi 🍓 ➜ ~ sudo usermod -a -G systemd-journal mermoldy
tail:
	@echo "Starring cat.hunter server on $(URL)..."
	ssh -t -p $(SSH_PORT) $(USER)@$(URL) "journalctl -u cat.hunter -f"
	@echo "Done"

clean:
	rm -rf targer
