URL=raspberrypi.local
INSTALL_DIR=/opt/cat.hunter
USER=mermoldy
SSH_PORT=22

SERVER_TARGET=armv7-unknown-linux-musleabihf

default: run

run:
	cargo fmt
	RUST_BACKTRACE=full RUST_LOG=debug cargo run

run_release:
	cargo fmt
	RUST_BACKTRACE=full RUST_LOG=debug cargo run --release

build:
	cargo fmt
	RUST_BACKTRACE=full RUST_LOG=debug cargo build

# Setup ARMv7 Toolchain for MacOS:
# - brew install arm-linux-gnueabihf-binutils
# - rustup target add armv7-unknown-linux-musleabihf
sync:
	@echo "Syncing cat.hunter files to $(URL):$(INSTALL_DIR)..."
	cargo fmt
	cargo +nightly build --release --workspace=server --bin=server --target=$(SERVER_TARGET) -Z config-profile
	rsync -e "ssh -p $(SSH_PORT)" Settings.toml "$(USER)@$(URL):$(INSTALL_DIR)"
	rsync -e "ssh -p $(SSH_PORT)" ./target/$(SERVER_TARGET)/release/server "$(USER)@$(URL):$(INSTALL_DIR)"
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
sync_run:
	@echo "Starring cat.hunter server on $(URL)..."
	ssh -t -p $(SSH_PORT) $(USER)@$(URL) "cd /opt/cat.hunter && sudo  ./server"
	@echo "Done"

# Setup:
# raspberrypi 🍓 ➜ ~ sudo usermod -a -G systemd-journal mermoldy
tail:
	@echo "Starring cat.hunter server on $(URL)..."
	ssh -t -p $(SSH_PORT) $(USER)@$(URL) "tail -f /var/log/cat.hunter.log -n 120"
	@echo "Done"

clean:
	rm -rf target
