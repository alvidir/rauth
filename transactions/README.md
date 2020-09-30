# Software design

## Diagram
The Session's service diagram is located at [Drive](https://drive.google.com/file/d/1ZWFn1SHSM5-B-NAJajIg7ZtEyoO0jzYy/view?usp=sharing)


## Transactions

| Name | Diagrams | Description |
|:-|:-:|:-|
| TxLogin | [Drive (TODO)]() | Creates a new session or update the latest one for a provided user, if exists. |
| TxGoogleLogin | [Drive (TODO)]() | Creates a new session or signs up a new user in base on google's provided information |
| TxLogout | [Drive (TODO)]() | Kills the provided session and all them related to the same user |
| TxSignup | [Drive (TODO)]() | Creates new user and opens its first session |

> Each transactions has its own implementation file located into `transactions folder`.