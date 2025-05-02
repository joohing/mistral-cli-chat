install:
	cargo build --release;
	sudo cp ./target/release/mistral-cli /usr/local/bin/mistral-cli
