# ksnotify

[![build](https://github.com/hirosassa/ksnotify/actions/workflows/test.yaml/badge.svg)](https://github.com/hirosassa/ksnotify/actions/workflows/test.yaml)
[![codecov](https://codecov.io/gh/hirosassa/ksnotify/branch/main/graph/badge.svg?token=IXWXVU95B8)](https://codecov.io/gh/hirosassa/ksnotify)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/hirosassa/ksnotify/blob/main/LICENSE)

A CLI command to parse `kubectl diff` result and notify it to GitLab/GitHub

## What ksnotify does

1. Parse the execution result of `kubectl diff`
1. Notify it to GitLab/GitHub

The comment posted by ksnotify is as follows:

------------
## Plan result

[CI link]( https://example.com )

* updated
  * apps.v1.Deployment.test.test-app

<details><summary>Details (Click me)</summary>

## apps.v1.Deployment.jasmine.test-app
```diff
 @@ -5,7 +5,6 @@
     deployment.kubernetes.io/revision: "3"
+  labels:
+    app: test-app
   name: test-app
   namespace: test
 spec:
@@ -27,7 +26,6 @@
       creationTimestamp: null
       labels:
         app: test-app
-        skaffold.dev/run-id: 1234
     spec:
       containers:
       - args:
```
</details>

------------

## Install

### GitHub Releases

Download the prebuilt binary from [GitHub Releases](https://github.com/hirosassa/ksnotify/releases) and install it to $PATH.

### aqua

Install ksnotify with [aqua](https://aquaproj.github.io/), which is a declarative CLI Version Manager.

```console
$ aqua g -i hirosassa/ksnotify
```

## Usage
### prerequisites

#### For GitLab

Create and export GitLab access token to environmental variables as follows:

```console
export KSNOTIFY_GITLAB_TOKEN="xxxxxx"
```
ref: [Project access tokens | GitLab](https://docs.gitlab.com/ee/user/project/settings/project_access_tokens.html)

#### For GitHub

If you run `ksnotify` on GitHub Actions, `ksnotify` use `GITHUB_TOKEN` by default.
If you run `ksnotify` locally, you should set PAT to `GITHUB_TOKEN` environment variable.

ref: [Permissions required for fine-grained personal access tokens](https://docs.github.com/en/rest/authentication/permissions-required-for-fine-grained-personal-access-tokens?apiVersion=2022-11-28)

If you simplify the configuration of setup `ksnotify` in GitHub Actions, you can use [setup-ksnotify](https://github.com/kitagry/setup-ksnotify).

### Post diff results to GitLab/GitHub

Basic usage for GitLab is as follows:

```console
skaffold render -p dev | kubectl diff -f - 2> /dev/null | ksnotify --ci gitlab
```

Of course, you can use `ksnotify` with GitHub as well.

```console
skaffold render -p dev | kubectl diff -f - 2> /dev/null | ksnotify --ci github
```

If you want to update the existing comment instead of create a new comment, you should add `--patch` flag like

```console
skaffold render -p dev | kubectl diff -f - 2> /dev/null | ksnotify --ci gitlab --patch
```

To suppress `skaffold` labels like `skaffold.dev/run-id: 1234` automatically added by `skaffold`, you should add `--suppress-skaffold` flag like

```console
skaffold render -p dev | kubectl diff -f - 2> /dev/null | ksnotify --ci gitlab --suppress-skaffold
```

The concrete example of GitLab CI configuration is shown in [example](https://github.com/hirosassa/ksnotify/tree/main/example).


## For developers

To run `ksnotify` locally, use local option for debug.
For local mode, `ksnotify` just renders contents on stdout.

```console
skaffold render -p dev | kubectl diff -f - 2> /dev/null | path/to/ksnotify --ci local --suppress-skaffold

> ## Plan result
> [CI link](  )
>
> * updated
> blah
> blah
```
