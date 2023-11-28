all: preset build

clean:
	rm -rd .bbin/ || true
	rm build.sh || true

preset:
	cd transpiler && cargo run -- ../preset.yaml

build:
	bash ./build.sh

