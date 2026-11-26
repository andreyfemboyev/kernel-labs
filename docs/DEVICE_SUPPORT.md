# Devices and emulators

Install ADB or Fastboot from Android platform tools. `klab device list` and `klab device list --fastboot` list discoverable devices. `klab device kernel SERIAL` and `klab device logs SERIAL` use ADB shell commands on the specified device. Diagnostic permissions depend on the device and build.

`klab emulator list` detects optional binaries. `klab emulator qemu-arm64 KERNEL INITRD --approve` attaches the terminal to a QEMU virt serial console with networking disabled. Use artifacts built for that machine and its PL011 console. The Cuttlefish and Android Emulator entries describe executable availability.
