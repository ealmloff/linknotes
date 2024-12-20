INSTALL_RUST := curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s --  -y

all:
	make linknotes

install:
	$(INSTALL_RUST)
	npm install

clean:
	cargo clean
	rm -f linknotes

linknotes: install src-tauri/src/*.rs
	npx tauri dev --features metal

test: ac
	cargo test

	@echo "---- ALL TESTS PASSED ---------"
