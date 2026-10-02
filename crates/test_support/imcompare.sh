#!/usr/bin/env bash

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]:-$0}")" && pwd)
TESTDIR=${1:?Specify testdir of failed images}
OS=${2:?Specify OS (macos or linux) of failed images}

tempdir=$(mktemp -d)
for f in $TESTDIR/*.png; do
    name=$(basename "$f")
    gm compare -highlight-style assign -metric PAE -maximum-error 0 "$ROOT/testdata/output/$OS/$name" "$f" -file "$tempdir/$name"
done

open -a Preview "$tempdir"
