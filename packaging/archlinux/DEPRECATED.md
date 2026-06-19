# DEPRECATED — do not install

This directory mirrors **upstream** `lianli-linux-git` packaging from
[sgtaziz/lian-li-linux](https://github.com/sgtaziz/lian-li-linux).

It installs `/usr/lib/modules-load.d/lianli-evdi.conf`, which loads the evdi
kernel module at **boot**. On NVIDIA + KDE Plasma Wayland that can cause a
black screen after the boot logo.

**Use whyLIAN instead:** `yay -S whylian` or `./scripts/install-dev.sh`

whyLIAN **does not** ship evdi, virtual-monitor support, or boot-time module
loading. HydroShift II AdvanceMode (fan, pump, LCD) works without evdi.
