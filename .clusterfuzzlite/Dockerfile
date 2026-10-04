FROM gcr.io/oss-fuzz-base/base-builder-rust@sha256:4d4bf15466807b053a811794c2ea980057e7b219010dcdcf7d6a1d4135a37d00
RUN cargo install cargo-fuzz --version 0.13.2 --locked --force -j 2
COPY . $SRC/diesel-builders
WORKDIR $SRC/diesel-builders
COPY ./.clusterfuzzlite/build.sh $SRC/
