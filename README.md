# rauth

[![Rust version](https://img.shields.io/badge/Rust-v1.54.0-orange.svg)](https://www.rust-lang.org/) [![tests](https://github.com/alvidir/rauth/actions/workflows/test.yaml/badge.svg?branch=master)](https://github.com/alvidir/rauth/actions/workflows/test.yaml)
[![rauth](https://img.shields.io/badge/rauth-v1.0.0-blue.svg)](https://github.com/alvidir/rauth)

Third-party authenticator

## About

Rauth is a third-party authentication for all Alvidir's applications. Deployed as a microservice, it will provide a session management for both sides, the user/client, and the application itself.

## Architecture

The whole service implementation as well as the files organization has been refactored in order to follow the [hexagonal architecture](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)) best practices. In this way, each object in the model got it's own folder in the _src_ directory where to find the following files:

| Name | Description |
|:-:|:-|
|mod| **[Required by Rust]** Declares the submodules in the directory and defines all these tests that ensures the module's robustness |
|framework| Implements the repository required by the `domain layer` as well as the gRPC service that exposes the endpoints for the object's use cases |
|application| Implements the use cases itself as callable functions totally independent of the _infrastructure/framework layer_ |
|domain| Declares the objects, relations and all its behaviours, as well as these interfaces/traits required by the objects itself |

## Design

The _conceptual diagram_ about rauth's model has been done via Draw.io provided by Google. The most up-to-date document can be found clicking [right here](https://drive.google.com/file/d/1huTe3jNqp3A_0WMB6tjhwSkBoqh_uA9F/view?usp=sharing).

The main objects of the domain and their role in the system are listed down below:

| Name | Description |
|:-:|:-|
| App | Represents an application any `User` can log in |
| User | Represents a physical person, or an entity, with a verifiable email able to log in into one or more `Apps` |
| Session | Represents a `User` who has logged in to at least one `App`|
| Directory | Represents the relation between a `User` and an `App`. A directory belongs to a `Session` and must have a Token associated to it|
| Token | It's de cookie itself, ensures the `Session` and `Directory` are easily findable by the system, and the data it represents reliable by the `App`'s host|
| Secret | Represents an array of bytes encoding a public or private key |
| Metadata | Represents a set of common attributes useful for management |

## Use cases
Use cases are usually translated as atomic methods the service's API exposes to clients. In the same way, each of the functionalities listed below corresponds to a transaction of the _application layer_ within the pertinent module, and independent of the rest.

| Name | Module | Description |
|:-:|:-:|:-|
| Register | App | Registers an `App` into the system, as well as its public key|
| Delete | App | Closes and deletes all `Directories` related to the `App`, removes the `App`'s `Secret` and finally unsubscribes the `App` from the system|
| Sign up | User | Registers a `User` into the system and sends a verification email to the provided email with an ephimeral `Token` for the verification process. |
| Verify | User | If, and only if, the provided `Token` is valid, the `User` gets verified and therefore granted for _Log In_ |
| Tfa | User | Enables or disables the two-factor-authenticator for user actions as __Log in__ and __Delete__ |
| Delete | User | Remove the `Session` and delete all `Directories` related to the `User`, removes the `User`'s `Secret` (if any) and finally unsubscribe the `User` from the system|
| Log in | Session | If the `User` has no `Session` in the system it gets generated as well as the cookie related to it. If the `User`'s `Session` has no open `Directory` for the requested `App` it gets loaded or created. A new `Token` for the `Directory` is generated and provided as response for the current request |
| Log out | Session | Close and save all `Directories` related to the `Session` and finally unsubscribe the `Session` from the system |

> The endpoints for the _use cases_ above are being implemented using [gRPC](https://grpc.io/) and [protocol buffer](https://developers.google.com/protocol-buffers)
