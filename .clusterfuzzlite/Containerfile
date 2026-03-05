FROM gcr.io/oss-fuzz-base/base-builder-rust
RUN apt-get update && apt-get install -y make autoconf automake libtool
COPY . $SRC/project
WORKDIR $SRC/project
COPY .clusterfuzzlite/build.sh $SRC/
