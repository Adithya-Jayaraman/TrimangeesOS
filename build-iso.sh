#!/usr/bin/env bash
# ============================================================
# Trimangees OS — Live ISO Build Script
# Run on Ubuntu 22.04+ with sudo
# Produces: trimangees-os.iso (~900MB)
# ============================================================
set -e
SCRIPT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD="$SCRIPT/build"
CHROOT="$BUILD/chroot"
ISO_OUT="$SCRIPT/trimangees-os.iso"

echo "==> Installing build tools"
apt-get update -qq
apt-get install -y debootstrap squashfs-tools xorriso \
    grub-pc-bin grub-efi-amd64-bin mtools

echo "==> Bootstrapping Debian Bookworm"
rm -rf "$CHROOT"
mkdir -p "$CHROOT"
debootstrap --arch=amd64 bookworm "$CHROOT" http://deb.debian.org/debian

echo "==> Installing packages inside chroot"
chroot "$CHROOT" /bin/bash << 'CHROOT_EOF'
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y \
    lightdm lightdm-gtk-greeter \
    xorg x11-xserver-utils xinit \
    openbox \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    fonts-inter \
    curl wget \
    nodejs npm \
    alsa-utils pulseaudio \
    network-manager \
    passwd \
    bash-completion \
    live-boot live-tools

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
source /root/.cargo/env

useradd -m -s /bin/bash -G sudo,audio,video,netdev adithya
echo "adithya:trimangees" | chpasswd
echo "root:root" | chpasswd

mkdir -p /etc/lightdm
cat > /etc/lightdm/lightdm.conf << 'LDM'
[SeatDefaults]
autologin-user=adithya
autologin-user-timeout=0
user-session=trimangees
LDM

systemctl enable lightdm NetworkManager
CHROOT_EOF

echo "==> Copying project into chroot"
mkdir -p "$CHROOT/opt/trimangees"
cp -r "$SCRIPT/shell"   "$CHROOT/opt/trimangees/"
cp -r "$SCRIPT/browser" "$CHROOT/opt/trimangees/"

echo "==> Building Trimangees Shell"
chroot "$CHROOT" /bin/bash << 'BUILD_EOF'
source /root/.cargo/env
cd /opt/trimangees/shell
cargo build --release
cp target/release/trimangees-shell /usr/bin/trimangees-shell
chmod +x /usr/bin/trimangees-shell
cd /opt/trimangees/browser
npm install
BUILD_EOF

echo "==> Installing session file"
mkdir -p "$CHROOT/usr/share/xsessions"
tee "$CHROOT/usr/share/xsessions/trimangees.desktop" << 'SESSION'
[Desktop Entry]
Name=Trimangees OS
Comment=Trimangees Desktop Shell
Exec=/usr/bin/trimangees-shell
Type=XSession
SESSION

echo "==> Setting environment variables"
tee "$CHROOT/etc/profile.d/trimangees.sh" << 'ENV'
export TRIMANGEES_BROWSER_DIR=/opt/trimangees/browser
export TRIMANGEES_ASSETS_DIR=/opt/trimangees/shell/assets/apps
ENV

echo "==> Configuring autostart"
mkdir -p "$CHROOT/home/adithya/.config/openbox"
tee "$CHROOT/home/adithya/.xinitrc" << 'XINIT'
#!/bin/bash
source /etc/profile.d/trimangees.sh
exec /usr/bin/trimangees-shell
XINIT
chmod +x "$CHROOT/home/adithya/.xinitrc"
chown -R 1000:1000 "$CHROOT/home/adithya"

echo "==> Building squashfs (this takes a while)"
mkdir -p "$BUILD/iso/live" "$BUILD/iso/boot/grub"
mksquashfs "$CHROOT" "$BUILD/iso/live/filesystem.squashfs" \
    -e boot -comp xz -Xbcj x86 -b 1M

echo "==> Copying kernel + initrd"
KVER=$(ls "$CHROOT/boot/vmlinuz-"* | sort | tail -1 | xargs basename | sed 's/vmlinuz-//')
cp "$CHROOT/boot/vmlinuz-$KVER"    "$BUILD/iso/boot/vmlinuz"
cp "$CHROOT/boot/initrd.img-$KVER" "$BUILD/iso/boot/initrd"

echo "==> Writing GRUB config"
tee "$BUILD/iso/boot/grub/grub.cfg" << 'GRUB'
set default=0
set timeout=3

menuentry "Trimangees OS" {
    linux  /boot/vmlinuz boot=live quiet splash
    initrd /boot/initrd
}

menuentry "Trimangees OS (debug)" {
    linux  /boot/vmlinuz boot=live
    initrd /boot/initrd
}
GRUB

echo "==> Building ISO"
grub-mkrescue -o "$ISO_OUT" "$BUILD/iso" -- -volid "TrimangeesOS" 2>/dev/null

echo ""
echo "========================================"
echo " Done! ISO: $ISO_OUT"
echo " VirtualBox: New VM → Linux 64-bit → attach ISO → boot"
echo " USB: dd if=$ISO_OUT of=/dev/sdX bs=4M status=progress"
echo "========================================"