# rauth

[![Rust version](https://img.shields.io/badge/Rust-v1.58.0-orange.svg)](https://www.rust-lang.org/) [![tests](https://github.com/alvidir/rauth/actions/workflows/test.yaml/badge.svg?branch=master)](https://github.com/alvidir/rauth/actions/workflows/test.yaml)
[![rauth](https://img.shields.io/badge/rauth-v1.0.0-blue.svg)](https://github.com/alvidir/rauth)

A simple SSO implementation in Rust 

## About

The Rauth project provides a **SSO** (Single Sign On) implementation that can be consumed as any of both, a [Rust](https://www.rust-lang.org/) library or a [gRPC](https://grpc.io/) service. Currently, the project includes all regular session-related actions as singup, login, logout, an so on. Plus **TOTP**(Time-based One Time Password) and email verification support.


## Endpoints

### Signup

| **Code** | Name | Description |
|:---------|:-----|:------------|
**E-001**|ERR_UNKNOWN| Unprevisible errors
**E-002**|ERR_NOT_FOUND| Token header not found
**E-003**|ERR_NOT_AVAILABLE| Require email verification
**E-005**|ERR_INVALID_TOKEN| Token is invalid because of any of the following reasons: bad format, `exp` time exceeded, bad signature, `nbf` not satisfied, wrong `knd` or not catched.
**E-006**|ERR_INVALID_FORMAT| Invalid format for `email` or `password`
**E-007**|ERR_INVALID_HEADER| Token header must be encoded in base64

### Reset

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


### Delete

| **Code** | Name | Description |
|:---------|:-----|:------------|
**E-001**|ERR_UNKNOWN| Unprevisible errors
**E-002**|ERR_NOT_FOUND| Token header not found
**E-004**|ERR_UNAUTHORIZED| Totp required
**E-005**|ERR_INVALID_TOKEN| Token is invalid because of any of the following reasons: bad format, `exp` time exceeded, bad signature, `nbf` not satisfied, wrong `knd` or not catched.
**E-007**|ERR_INVALID_HEADER| Token header must be encoded in base64
**E-008**|ERR_WRONG_CREDENTIALS| Password does not match or invalid `user id`.

### Totp

| **Code** | Name | Description |
|:---------|:-----|:------------|
**E-001**|ERR_UNKNOWN| Unprevisible errors
**E-002**|ERR_NOT_FOUND| Token header not found
**E-003**|ERR_NOT_AVAILABLE| The action cannot be performed
**E-004**|ERR_UNAUTHORIZED| Invalid `totp` value
**E-005**|ERR_INVALID_TOKEN| Token is invalid because of any of the following reasons: bad format, `exp` time exceeded, bad signature, `nbf` not satisfied, wrong `knd` or not catched.
**E-007**|ERR_INVALID_HEADER| Token header must be encoded in base64
**E-008**|ERR_WRONG_CREDENTIALS| Password does not match or invalid `user id`.

### Login

| **Code** | Name | Description |
|:---------|:-----|:------------|
**E-001**|ERR_UNKNOWN| Unprevisible errors
**E-004**|ERR_UNAUTHORIZED| Totp required
**E-008**|ERR_WRONG_CREDENTIALS| Invalid `username` or `password`

### Logout

| **Code** | Name | Description |
|:---------|:-----|:------------|
**E-001**|ERR_UNKNOWN| Unprevisible errors
**E-002**|ERR_NOT_FOUND| Token header not found
**E-005**|ERR_INVALID_TOKEN| Token is invalid because of any of the following reasons: bad format, `exp` time exceeded, bad signature, `nbf` not satisfied, wrong `knd` or not catched.
**E-007**|ERR_INVALID_HEADER| Token header must be encoded in base64
**E-008**|ERR_WRONG_CREDENTIALS| Invalid `username` or `password`
