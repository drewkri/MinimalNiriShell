# MinimalNiriShell

A minimal desktop shell setup designed for the niri compositor and my own personal tastes.  
Integrates heavily with Vicinae for most tasks, and some other applications are used to cover other bases.

# Features

- Custom Vicinae plugin to display battery, interface with the system tray, media players, wifi, bluetooth, and more
- Hot corner in one corner of the desktop to open this plugin's interface (defaults to top-right, customizable via cli arg)
- Bar designed for tablets/2-in-1 devices to make using the desktop easier without a keyboard (works fine on desktops too)

# Notes

- Some buttons in the Vicinae quick menu are designed specifically for systemd systems and may not work on other init systems.
- The shortcut to open advanced network settings opens nm-connection-editor by default, you may need to change this in the preferences if you do not use NetworkManager.
- The quick menu is configured not to show battery by default, but if you are on a laptop, you may want to enable this in the extension preferences.

# Installation

Clone the repository and run `setup.sh` (TODO: Make this work).

In order for everything to work, you must have the following packages installed (you can probably find all of them in your distro's packages):

- cargo
- npm
- playerctl
- FontAwesome nerd fonts (used for the bar's battery display)

The following applications are used by default with quick-menu, although you can replace them with something else in the preferences
if you prefer:

- nm-connection-editor
- swaync

The script will do the following:

- Install Vicinae (if not installed) to ~/.local and start the server
- Install the quick menu extension (via npm)
- Compile and install the rust projects in this repository, vicinae-shell-integration, to ~/.local/bin
- Prompt you to install several Vicinae extensions

You will need to bring your own notification daemon. The recommended default is swaync, if you wish to use another notification daemon
you will need to change the settings of the quick menu extension to make the notifications button open the correct program.

Then, edit your Niri configuration and make sure it spawns Vicinae (`vicinae server`) and the Rust glue program for system tray support and the hot corner (`vicinae-shell-integration`). It has some command line options you may want to review with `vicinae-shell-integration --help` to
handle a desktop background and customize the size and position of the hot corner. If you want more background options consider bringing
your own program e.g. swaybg.

Optionally, have the device spawn the bar (`niri-vicinae-bar`). You can also have the device spawn the bar only when tablet mode is detected, and kill it (`pkill niri-vicinae-bar`) when the device is no longer in tablet mode.

The bar has some settings which can be configured by running `niri-vicinae-bar --settings`. It will also attempt to load custom CSS from
a file named `style.css` in the config directory (`~/.config/io.github.drewkri.niri-vicinae-bar`).

You can then safely delete the repository and adjust settings as desired.
