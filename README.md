# oauth

Third-party authenticator

## About

Oauth is a third-party authentication for all Alvidir's applications. Deployed as a microservice, it will provide a session management for both sides, the user/client, and the application itself.

## Architecture

The latest version of this service includes a structure refactoring in order to follow the [hexagonal architecture](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)) best practices

## Design

The `conceptual diagram` about oauth's model has been done via `Draw.io` provided by Google. The most up-to-date document could be found clicking [right here](https://drive.google.com/file/d/1huTe3jNqp3A_0WMB6tjhwSkBoqh_uA9F/view?usp=sharing).

The main objects of the domain and their role in the system are listed down below:

| Name | Description |
|:-:|:-|
| App | Represents an application any `User` can log in |
| User | Represents a physical person, or an entity, with a verifiable email able to log in into one or more `Apps` |
| Session | Represents an `User` who has logged in to at least one `App`|
| Directory | Represents the relation between a `User` and an `App`. A directory belongs to a `Session` and must have a Token associated to it|
| Token | It's de cookie itself, ensures the `Session` and `Directory` are easily findable by the system, and the data it represents reliable by the `App`'s host|
| Secret | Represents an array of bytes encoding a private key |
| Metadata | Represents a set of common attributes useful for management |

## Use cases

| Name | Subject | Description |
|:-:|:-:|:-|
| Register | App | Register an `App` into the system, as well as its public key|
| Delete | App | Close and delete all `Directories` related to the `App`, removes the `App`'s `Secret` and finally unsubscribe the `App` from the system|
| Sign up | User | Register an ephemeral `User` into the system and send a verification email to the provided email. Once the email got confirmed, the `User` becomes persistent and its `Secret` gets generated and registered |
| Delete | User | Close and delete all `Directories` related to the `User`, removes the `User`'s `Secret` (if any) and finally unsubscribe the `User` from the system|
| Log in | Session | If the `User` has no `Session` in the system it gets generated as well as the cookie related to it. If the `User`'s `Session` has no open `Directory` for the requested `App` it gets loaded or created. A new `Token` for the `Directory` is generated and provided as response for the current request |
| Log out | Session | Close and save all `Directories` related to the `Session` and finally unsubscribe the `Session` from the system |

> The endpoints for the `Use Cases` above are being implemented using [gRPC](https://grpc.io/) and [protocol buffer](https://developers.google.com/protocol-buffers)