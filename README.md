# ksnotify

[![build](https://github.com/hirosassa/ksnotify/actions/workflows/test.yaml/badge.svg)](https://github.com/hirosassa/ksnotify/actions/workflows/test.yaml)
[![codecov]()](https://codecov.io/gh/hirosassa/ksnotify)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/hirosassa/ksnotify/blob/main/LICENSE)

A CLI command to parse `kubectl diff` result and notify it to GitLab

## Caution

This repository is under development status.

## What ksnotify does

1. Parse the execution result of `kubectl diff`
1. Notify it to GitLab

## Usage

Basic usage is as follows:

```console
kustomize build dev | kubectl diff -f - 2> /dev/null | | ksnotify
```

To suppress `skaffold` labels like `skaffold.dev/run-id: 1234` automatically added by `skaffold`, you should declare

```console
export KSNOTIFY_SUPPRESS_SKAFFOLD=1
```
