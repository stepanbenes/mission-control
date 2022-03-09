FROM rustembedded/cross:armv7-unknown-linux-gnueabihf-0.2.1

RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get install --assume-yes pkg-config:armhf libudev-dev:armhf libssl-dev:armhf && \
    export PKG_CONFIG_PATH=/usr/lib/pkgconfig
