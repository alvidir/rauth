# oauth

Third-party authenticator

## Demos 

This project is being developed in `Go` and `Rust` simultaneously. Each of them has its own branch in order to keep the project's coherence. Such branches are prefixed by the keyword `demo` and followed by the language name (using the character `-` between words).

At today's date 10/01/2021 (dd/mm/yyyy) these branches' status are the following:

| Brach | Source | Status |
|:-|:-:|:-|
| demo-rust | `rust` | The model objects and its persistence are being implemented through the `diesel` engine for **Postgres** management. At this point, signup and login transactions for clients of the User kind are working properly and ready for scalation. These login can be done via name or email. The `next steps` are to implement the ephimeral token generation and the long-term token activation through such ephimeral tokens from the app side.|
| demo-go | `go` | Sign In with google is implemented, as well as the dummy user for testing and debugging. Persistence management is being done via `xorm` to communicate with the **mysql** database. |

## Design

The `conceptual diagram` about oauth's model has been done via `Draw.io` provided by Google.

Down below are listed each diagram and provided its public links. There must be described all agents involved and its cardinalities as well as its navigavilities. 

| Name | Diagrams | Description |
|:-|:-:|:-|
| Model | [Drive](https://drive.google.com/file/d/1huTe3jNqp3A_0WMB6tjhwSkBoqh_uA9F/view?usp=sharing) | oauth's service agents and how do they interact with each other |

## Ojects

| Name | Description |
|:-|:-|
| Client | A Client object relates a session with the client data it belongs |
| App | An App is the kind of client that represents an application users can log in |
| User | A User is the kind of client that represents a user able to login into one or more applications |
| Session | A Sessions allows any client to signin into an app |
