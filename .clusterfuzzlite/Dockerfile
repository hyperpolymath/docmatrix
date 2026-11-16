# SPDX-License-Identifier: MPL-2.0
# Verified registry manifest, 2026-11-16.
FROM gcr.io/oss-fuzz-base/base-builder-rust@sha256:3428d0a90b1e7efbe2ba947bd1d0c9a70594986b4b6553804547f1fb24cfc92f
RUN apt-get update && apt-get install -y make autoconf automake libtool
COPY . $SRC/project
WORKDIR $SRC/project
COPY .clusterfuzzlite/build.sh $SRC/
