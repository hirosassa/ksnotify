# ksnotify

A CLI command to parse kustomize build result and notify it to GitLab

## Caution

This repository is under development status.

## What ksnotify does

1. Parse the execution result of `Kustomize`
2. Bind parsed results to handlebars templates
3. Notify it to GitLab as you like

## Usage

```console
kustomize build | ksnotify
```
