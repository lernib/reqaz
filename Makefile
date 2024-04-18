OLD_CLI_DIR=cli
OLD_CLI_MAIN=$(abspath $(OLD_CLI_DIR)/build/index.js)
OLD_CLI_SOURCES=\
	$(OLD_CLI_DIR)/package.json \
	$(OLD_CLI_DIR)/tsconfig.json \
	$(shell find $(OLD_CLI_DIR)/src -type f -name '*.ts')


website: dep_website
	cargo run -p nib-website-cli -- $(PWD)/website --framework

old_cli: dep_old_cli
	node $(OLD_CLI_MAIN)

cli: dep_cli

dep_website: dep_cli

dep_old_cli: $(OLD_CLI_MAIN)

dep_cli:

$(OLD_CLI_MAIN): $(OLD_CLI_SOURCES)
	cd cli && npm run build
