# tp-auth

This repo has been migrated from the [Mastermind project](https://github.com/alvidir/mastermind).

Due its potential seems appropriate to detach it from any other project and keep it as an independent repository; this means upgrading this utilitie as a brand new `service/package`; being reachable for all this applications that require any kind of authentication management.

## Demos 

This project is being developed in `Go` and `Rust` simultaneously. Each of them has its own branch in order to keep the project's coherence. Such branches are prefixed by the keyword `demo` and followed by the language name (using the character `-` between words).

At today's date 09/12/2020 (dd/mm/yyyy) these branches' status are the following:

| Brach | Source | Status |
|:-|:-:|:-|
| demo-rust | `rust` | The model objects and its persistence are being implemented through the `diesel` engine for **Postgres** management. |
| demo-go | `go` | Sign In with google is implemented, as well as the dummy user for testing and debugging. Persistence management is being done via `xorm` to communicate with the **mysql** database. |

## Design

To learn more about this project there are available a set of documents to conceptualize the main-stream for this repo.

| Name | Path | Description |
|:-|:-:|:-|
| Model | [Readme](./src/models/README.md) | System UML design |
