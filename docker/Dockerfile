FROM ubuntu:22.04

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y \
        gawk \
        wget \
        git-core \
        subversion \
        diffstat \
        unzip \
        sysstat \
        texinfo \
        gcc-multilib \
        build-essential \
        gcc-11 \
	gcc-11-base \
	chrpath \
        socat \
        file \
	python-is-python3 \
        python3-pip \
        xz-utils  \
        locales \
        cpio \
        screen \
        tmux \
        sudo \
        iputils-ping \
        iproute2 \
        fluxbox \
        tig \
        vim \
        jq \
        apt-transport-https \
        ca-certificates \
        curl \
        gnupg \
        lsb-release \
        chrpath \
        diffstat \
        g++ \
        make \
        lz4 \
        zstd \
        python3-setuptools

RUN curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg && \
    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu \
    $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null

RUN apt-get update && \
    apt-get install -y \
        docker-ce \
        docker-ce-cli \
        containerd.io

RUN dpkg-reconfigure locales && \
    locale-gen en_US.UTF-8 && \
    update-locale LC_ALL=en_US.UTF-8 LANG=en_US.UTF-8

ENV LC_ALL en_US.UTF-8
ENV LANG en_US.UTF-8
ENV LANGUAGE en_US.UTF-8

WORKDIR /bakery
COPY ../../artifacts/bakery.deb .
RUN dpkg -i bakery.deb

# Setting up a bakery specific bash env pulled in by /etc/bash.bashrc 
RUN mkdir -p /etc/bakery && \
     echo "source /etc/bakery/bakery.bashrc" >> /etc/bash.bashrc

RUN rm /bin/sh && \
    ln -s /bin/bash /bin/sh
