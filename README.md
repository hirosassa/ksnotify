# ksnotify

[![build](https://github.com/hirosassa/ksnotify/actions/workflows/test.yaml/badge.svg)](https://github.com/hirosassa/ksnotify/actions/workflows/test.yaml)
[![codecov](https://codecov.io/gh/hirosassa/ksnotify/branch/main/graph/badge.svg?token=IXWXVU95B8)](https://codecov.io/gh/hirosassa/ksnotify)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/hirosassa/ksnotify/blob/main/LICENSE)

A CLI command to parse `kubectl diff` result and notify it to GitLab

## What ksnotify does

1. Parse the execution result of `kubectl diff`
1. Notify it to GitLab

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


## Usage

Basic usage is as follows:

```console
kustomize build dev | kubectl diff -f - 2> /dev/null | | ksnotify
```

To suppress `skaffold` labels like `skaffold.dev/run-id: 1234` automatically added by `skaffold`, you should declare

```console
export KSNOTIFY_SUPPRESS_SKAFFOLD=1
```
