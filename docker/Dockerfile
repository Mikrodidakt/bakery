FROM ubuntu:20.04

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
        chrpath \
        socat \
        python \
        python3.7 \
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
        lsb-release

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

RUN pip3 install pipenv pre-commit

COPY Pipfile* /tmp/
RUN cd /tmp/ && pipenv lock --dev --requirements > requirements.txt
RUN pip3 install -r /tmp/requirements.txt

COPY . /tmp/bakery
RUN pip3 install /tmp/bakery/dist/bakery-*.whl

RUN rm /bin/sh && \
    ln -s /bin/bash /bin/sh
