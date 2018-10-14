FROM base/devel

RUN curl https://sh.rustup.rs -sSf | sh -s  -- -y

RUN useradd builder
RUN pacman -Sy --noconfirm git sudo fakeroot