# rauth

[![Rust version](https://img.shields.io/badge/Rust-v1.58.0-orange.svg)](https://www.rust-lang.org/) [![tests](https://github.com/alvidir/rauth/actions/workflows/test.yaml/badge.svg?branch=master)](https://github.com/alvidir/rauth/actions/workflows/test.yaml)
[![rauth](https://img.shields.io/badge/rauth-v1.0.0-blue.svg)](https://github.com/alvidir/rauth)

A simple SSO implementation in Rust 

## About

The rauth project provides a **SSO** (Single Sign On) implementation that can be consumed as any of both, a [Rust](https://www.rust-lang.org/) library or a [gRPC](https://grpc.io/) service. Currently, the project includes all regular session-related actions as signup, login, logout, and so on. Plus **TOTP**(Time-based One Time Password) and email verification support.

## Setup

To get the environment ready for the application to run, several steps have to be completed. Luckily all commands are in the [Makefile](./Makefile) of this project, so don't panic :P

It is required to include in a `.env` file the following environment variables, since the [db setupt script](./scripts/build_db_setup_script.py) requires them.
``` bash
DB_MIGRATIONS_PATH=./migrations
DB_SETUP_SCRIPT_PATH=./migrations/.postgres/setup.sql
DB_FILE_REGEX=up.sql
```

Running the following command in your terminal will be created an sql setup script at `migrations/.postgres` as well as a `.ssh` directory where to find the JWT keypair required by the server:
``` bash
$ make setup
```

If you also expect to deploy the application using `podman` build the rauth image by executing:
``` bash
$ make build
```

Last but not least, the service will expect a directory (`templates` by default) with the following email templates:

| Filename | Description |
|:---------|:------------|
`verification_email.html`| The html template to render and send when an email has to be verified.
`reset_email.html` | The html template to render and send when a user requests for resetting its password.

> Both templates may consume the same variables: `name` and `token`, provided by the server while rendering. 

## Configuration

The server expects a set of environment variables to work properly. Although some of them are optional, it is recommended to assign a value to all of them to have absolute awareness about how the service will behave.

This is an example of a `.env` that can be used in a deployment:
```bash
# service settings
SERVICE_PORT=8000
SERVICE_NETW=0.0.0.0

# postgres settings
POSTGRES_DB=rauth
POSTGRES_USER=admin
POSTGRES_PASSWORD=adminpwd
POSTGRES_POOL=1

# postgres dsn
DATABASE_URL=postgres://admin:adminpwd@rauth-postgres/rauth?sslmode=disable&connect_timeout=1

# redis settings
REDIS_HOSTNAME=rauth-redis
REDIS_DSN=redis://rauth-redis:6379/?
REDIS_POOL=1

# security settings
TOKEN_TIMEOUT=7200
JWT_SECRET=<content of .ssh/pkcs8_key.base64 file generated on the setup step>
JWT_PUBLIC=<content of .ssh/pkcs8_pubkey.base64 file generated on the setup step>
JWT_HEADER=authorization
TOTP_HEADER=x-totp-secret

# email settings
SMTP_ISSUER=rauth
SMTP_ORIGIN=no-reply@alvidir.com
SMTP_TRANSPORT=mailhog:1025
SMTP_TEMPLATES=./templates

# [i] not setting a username nor password for smtp means disabling the TLS for the smtp transport
# SMTP_USERNAME=<your username>
# SMTP_PASSWORD=<the application password>

# in order to use google gmail as smtp, it is required to create an application password on
# our account. See: https://support.google.com/accounts/answer/185833
```

## Deployment

Since the application needs some external services to be launched, the easiest way to deploy them all is by using `podman-compose` as following:
```bash
$ make deploy
```

This command will deploy a pod with all those services described in the [compose file](./docker-compose.yaml) of this project. Once completed, the application endpoints will be reachable in two different ways:
- via grpc messaging on port `8000`
- via grpc-web requests on port `8080`

## Logs

By default, the deployment command has the `-d` flag enabled, so no logs are displayed. If you really want to see them, you have two options: removing the `-d` flag from the `deploy` command of the [Makefile](./Makefile), which will display all logs of all services, or running the following command to display only those coming from the `rauth-server`:
```bash
$ make follow
```

## Endpoints
### **Signup**

Allows a new user to get registered into the system if, and only if, `email` and `password` are both valid. The latter does not only refer to format, but also the `email` is verifiable.

### Request

The **signup** transaction requires of two steps to get completed: the _signup request_, and the _email verification_. Both of them use the exact same endpoint to get performed, nonetheless, the _signup request_ is the only one that must all fields. The _email verification_ instead, must provide the verification token in the corresponding header.

``` yaml
# An example of a gRPC message for signup

{
    "email": "dummy@test.com" # an string containing the user's email,
    "pwd": "1234567890ABCDEF" # an string containing the user's password encoded in base64
}
```
> If, and only if, the email verification completed successfully, an Empty response is sent with the session token in the corresponding header 

### Errors

| **Code** | Name | Description |
|:---------|:-----|:------------|
**E-001**|ERR_UNKNOWN| Unprevisible errors
**E-002**|ERR_NOT_FOUND| Token header not found
**E-003**|ERR_NOT_AVAILABLE| Require email verification
**E-005**|ERR_INVALID_TOKEN| Token is invalid because of any of the following reasons: bad format, `exp` time exceeded, bad signature, `nbf` not satisfied, wrong `knd` or not catched.
**E-006**|ERR_INVALID_FORMAT| Invalid format for `email` or `password`
**E-007**|ERR_INVALID_HEADER| Token header must be encoded in base64

### **Reset**

| **Code** | Name | Description |
|:---------|:-----|:------------|
**E-001**|ERR_UNKNOWN| Unprevisible errors
**E-002**|ERR_NOT_FOUND| Token header not found
**E-003**|ERR_NOT_AVAILABLE| Require email verification
**E-004**|ERR_UNAUTHORIZED| Totp required
**E-005**|ERR_INVALID_TOKEN| Token is invalid because of any of the following reasons: bad format, `exp` time exceeded, bad signature, `nbf` not satisfied, wrong `knd` or not catched.
**E-006**|ERR_INVALID_FORMAT| Password must be encoded in base64
**E-007**|ERR_INVALID_HEADER| Token header must be encoded in base64
**E-008**|ERR_WRONG_CREDENTIALS| The new password cannot match the old one or invalid `user id`.


### **Delete**

| **Code** | Name | Description |
|:---------|:-----|:------------|
**E-001**|ERR_UNKNOWN| Unprevisible errors
**E-002**|ERR_NOT_FOUND| Token header not found
**E-004**|ERR_UNAUTHORIZED| Totp required
**E-005**|ERR_INVALID_TOKEN| Token is invalid because of any of the following reasons: bad format, `exp` time exceeded, bad signature, `nbf` not satisfied, wrong `knd` or not catched.
**E-007**|ERR_INVALID_HEADER| Token header must be encoded in base64
**E-008**|ERR_WRONG_CREDENTIALS| Password does not match or invalid `user id`.

### **Totp**

| **Code** | Name | Description |
|:---------|:-----|:------------|
**E-001**|ERR_UNKNOWN| Unprevisible errors
**E-002**|ERR_NOT_FOUND| Token header not found
**E-003**|ERR_NOT_AVAILABLE| The action cannot be performed
**E-004**|ERR_UNAUTHORIZED| Invalid `totp` value
**E-005**|ERR_INVALID_TOKEN| Token is invalid because of any of the following reasons: bad format, `exp` time exceeded, bad signature, `nbf` not satisfied, wrong `knd` or not catched.
**E-007**|ERR_INVALID_HEADER| Token header must be encoded in base64
**E-008**|ERR_WRONG_CREDENTIALS| Password does not match or invalid `user id`.

### **Login**

| **Code** | Name | Description |
|:---------|:-----|:------------|
**E-001**|ERR_UNKNOWN| Unprevisible errors
**E-004**|ERR_UNAUTHORIZED| Totp required
**E-008**|ERR_WRONG_CREDENTIALS| Invalid `username` or `password`

### **Logout**

| **Code** | Name | Description |
|:---------|:-----|:------------|
**E-001**|ERR_UNKNOWN| Unprevisible errors
**E-002**|ERR_NOT_FOUND| Token header not found
**E-005**|ERR_INVALID_TOKEN| Token is invalid because of any of the following reasons: bad format, `exp` time exceeded, bad signature, `nbf` not satisfied, wrong `knd` or not catched.
**E-007**|ERR_INVALID_HEADER| Token header must be encoded in base64
**E-008**|ERR_WRONG_CREDENTIALS| Invalid `username` or `password`