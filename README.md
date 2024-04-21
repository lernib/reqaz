# reqaz

Requests from A to Z (reqaz) is a tool to help manage varions aspects of static HTML pages. We at Lernib use it to help bundle things like CSS and certain HTML assets ahead of time before deploying to a bucket.

This tool is not stable, use at your own risk.

# Usage (CLI)

Install the CLI with Cargo:

```shell
cargo install --git https://github.com/lernib/reqaz
```

Add a reqaz.json to your project for ease of mind:

```json
{
    "root": "public",
    "port": 5000,
    "log": true,
    "generate": {
        "output_dir": ".reqaz/build",
        "pipelines": [
            {
                "input": "/",
                "output": "index.html"
            }
        ]
    }
}
```

Run reqaz to build all specified pipelines:

```shell
reqaz
```

Alternatively, you can use reqaz as a dev server (no hot reloading yet):

```shell
reqaz serve
```

# Usage (library)

This package is not ready for use as a library yet. Once that is ready, docs will be added here.
