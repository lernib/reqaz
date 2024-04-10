CLI_DIR=cli
CLI_MAIN=$(abspath $(CLI_DIR)/build/index.js)
CLI_SOURCES=\
	$(CLI_DIR)/package.json \
	$(CLI_DIR)/tsconfig.json \
	$(shell find $(CLI_DIR)/src -type f -name '*.ts')

website: dep_website
	cd website && node $(CLI_MAIN)

cli: dep_cli
	node $(CLI_MAIN)

dep_website: dep_cli

dep_cli: $(CLI_MAIN)

$(CLI_MAIN): $(CLI_SOURCES)
	cd cli && npm run build
