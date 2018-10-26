FROM base/devel

RUN useradd builder
RUN pacman -Sy --noconfirm git sudo fakeroot

RUN curl https://sh.rustup.rs -sSf | sh -s  -- -y