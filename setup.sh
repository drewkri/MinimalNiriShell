#!/usr/bin/env bash

cd "${BASH_SOURCE[0]%/*}/niri-vicinae-bar"
cargo build --release
cp target/release/niri-vicinae-bar ~/.local/bin/niri-vicinae-bar
cd ../vicinae-shell-integration
cargo build --release
cp target/release/vicinae-shell-integration ~/.local/bin/vicinae-shell-integration

if vicinae; then
    echo "Vicinae already installed, skipping install"
else
    curl -fsSL https://vicinae.com/install | bash -s -- --prefix ~/.local
    vicinae serve &
fi
# install extensions
echo "Now you will be prompted to install several vicinae extensions."
echo "After each, close vicinae to proceed with installation."
sleep 3
vicinae vicinae://extensions/leonkohli/process-manager
while vicinae state open; do
    sleep 0.1
done
vicinae vicinae://extensions/gelei/bluetooth
while vicinae state open; do
    sleep 0.1
done
vicinae vicinae://extensions/dagimg-dot/wifi-commander
while vicinae state open; do
    sleep 0.1
done
vicinae vicinae://extensions/rastsislaux/pulseaudio
while vicinae state open; do
    sleep 0.1
done

echo "Finishing install, after you see \"Extension built\" you can close this terminal."
cd ../quick-menu
npm install
npm run dev
