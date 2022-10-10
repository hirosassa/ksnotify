# ksnotify

A CLI command to parse `kubectl diff` result and notify it to GitLab

## Caution

This repository is under development status.

## What ksnotify does

1. Parse the execution result of `kubectl diff`
1. Notify it to GitLab

## Usage

```console
kustomize build dev | kubectl diff -f - 2> /dev/null | | ksnotify
```
