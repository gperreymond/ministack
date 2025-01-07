ARG UBUNTU_JAMMY_IMAGE="ubuntu:jammy-20240911.1"

FROM ${UBUNTU_JAMMY_IMAGE} AS hashibase

ARG CNI_PLUGINS_VERSION="1.6.2"

# copy files
COPY scripts/start-ministack /usr/local/bin/start-ministack
COPY scripts/entrypoint /usr/local/bin/entrypoint
COPY scripts/clean-install /usr/local/bin/clean-install
COPY files/bridge.conf /etc/sysctl.d/bridge.conf

# Install dependencies, first from apt, then from release tarballs.
# NOTE: we use one RUN to minimize layers.
#
# First we must ensure that our util scripts are executable.
#
# The base image already has: ssh, apt, snapd, but we need to install more packages.
# Packages installed are broken down into (each on a line):
# - packages needed to run services (systemd)
# - packages needed for kubernetes components
# - packages needed by the container runtime
# - misc packages kind uses itself
# - packages that provide semi-core kubernetes functionality
# After installing packages we cleanup by:
# - removing unwanted systemd services
# - disabling kmsg in journald (these log entries would be confusing)
#
# Finally we adjust tempfiles cleanup to be 1 minute after "boot" instead of 15m
# This is plenty after we've done initial setup for a node, but before we are
# likely to try to export logs etc.
RUN echo "Ensuring scripts are executable ..." \
  && chmod +x \
    /usr/local/bin/clean-install \
    /usr/local/bin/entrypoint \
    /usr/local/bin/start-ministack \
  && echo "Installing Packages ..." \
    && DEBIAN_FRONTEND=noninteractive clean-install \
      systemd \
      conntrack iptables iproute2 ethtool socat util-linux mount ebtables udev kmod \
      libseccomp2 pigz \
      bash ca-certificates curl rsync unzip \
      nfs-common \
      iputils-ping netcat-openbsd vim-tiny \
    && find /lib/systemd/system/sysinit.target.wants/ -name "systemd-tmpfiles-setup.service" -delete \
    && rm -f /lib/systemd/system/multi-user.target.wants/* \
    && rm -f /etc/systemd/system/*.wants/* \
    && rm -f /lib/systemd/system/local-fs.target.wants/* \
    && rm -f /lib/systemd/system/sockets.target.wants/*udev* \
    && rm -f /lib/systemd/system/sockets.target.wants/*initctl* \
    && rm -f /lib/systemd/system/basic.target.wants/* \
    && echo "ReadKMsg=no" >> /etc/systemd/journald.conf \
    && ln -s "$(which systemd)" /sbin/init \
  && echo "Adjusting systemd-tmpfiles timer" \
    && sed -i /usr/lib/systemd/system/systemd-tmpfiles-clean.timer -e 's#OnBootSec=.*#OnBootSec=1min#' \
  && echo "Disabling udev" \
    && systemctl disable udev.service \
  && echo "Modifying /etc/nsswitch.conf to prefer hosts" \
    && sed -i /etc/nsswitch.conf -re 's#^(hosts:\s*).*#\1dns files#' \
  && echo "Update bridge" \
    && echo 1 | sudo tee /proc/sys/net/bridge/bridge-nf-call-arptables \
    && echo 1 | sudo tee /proc/sys/net/bridge/bridge-nf-call-ip6tables \
    && echo 1 | sudo tee /proc/sys/net/bridge/bridge-nf-call-iptables

# delete this file due to https://github.com/kubernetes/minikube/issues/17700
RUN rm -f /usr/lib/binfmt.d/python3.10.conf

# install system requirements from the regular distro repositories
RUN clean-install \
    lz4 \
    gnupg \
    sudo \
    openssh-server \
    dnsutils \
    # libglib2.0-0 is required for conmon, which is required for podman
    libglib2.0-0

# install docker
RUN sh -c "echo 'deb https://download.docker.com/linux/ubuntu jammy stable' > /etc/apt/sources.list.d/docker.list" && \
    curl -L https://download.docker.com/linux/ubuntu/gpg -o docker.key && \
    apt-key add - < docker.key && \
    clean-install docker-ce docker-ce-cli containerd.io docker-buildx-plugin && \
    # enable docker which is default
    systemctl enable docker.service

# install containernetworking-plugins
RUN curl -LO "https://github.com/containernetworking/plugins/releases/download/v${CNI_PLUGINS_VERSION}/cni-plugins-linux-amd64-v${CNI_PLUGINS_VERSION}.tgz" && \
    mkdir -p /opt/cni/bin && \
    tar -xf "cni-plugins-linux-amd64-v${CNI_PLUGINS_VERSION}.tgz" -C /opt/cni/bin && \
    rm "cni-plugins-linux-amd64-v${CNI_PLUGINS_VERSION}.tgz"

# install hashiup
RUN curl -sLS https://get.hashi-up.dev | sh

# Deleting leftovers
RUN rm -rf \
  /usr/share/doc/* \
  /usr/share/man/* \
  /usr/share/local/* \
  /var/lib/apt/lists/*

# squash all layers into one
FROM scratch
COPY --from=hashibase / /

USER root

ENV NOMAD_VERSION="1.9.4"
ENV CONSUL_VERSION="1.20.1"
ENV VAULT_VERSION="1.18.3"

# tell systemd that it is in docker (it will check for the container env)
# https://systemd.io/CONTAINER_INTERFACE/
ENV container=docker
# systemd exits on SIGRTMIN+3, not SIGTERM (which re-executes it)
# https://bugzilla.redhat.com/show_bug.cgi?id=1201657
STOPSIGNAL SIGRTMIN+3

ENTRYPOINT [ "/usr/local/bin/entrypoint", "/sbin/init" ]
