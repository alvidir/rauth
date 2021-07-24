# oauth

Third-party authenticator

## About

Oauth is a third-party authentication for all Alvidir's applications. Deployed as a microservice, this will provide a session management for both sides, the user/client, and the application itself.

## Design

The `conceptual diagram` about oauth's model has been done via `Draw.io` provided by Google. [Clicking right here](https://drive.google.com/file/d/1huTe3jNqp3A_0WMB6tjhwSkBoqh_uA9F/view?usp=sharing) you will be redirected to the public document.

## Objects

| Name | Description |
|:-:|:-|
| User | Represents a physical person or an entity with a verifiable email able to log in into one or more `App`s |
| App | Represents an application any `User` can log in |
| Session | Represents a `User` who has logged in to at least one `App`|
| Directory | Represents the relation between a `User` and an `App`. A directory belongs to a `Session` and must have a Token associated to it|
| Token | It's de cookie itself, ensures the `Session` and `Directory` will be identifiable by the system and reliable by the `App`'s host|
| Secret | Represents a document with all these data required by the system to ensure actions security |
| Metadata | Represents a set of common data useful for management |

## API

The API for this application is being implemented using `protocol buffers`.