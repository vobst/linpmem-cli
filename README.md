# pmem

`pmem` is a small tool for loading and interacting with the [linpmem driver](). It lets you use the features of the driver in scripts and on the command line. At the same time, this repository also provides a library that can be used by other programs that want to interface with the driver. The command-line application is simply a thin wrapper around this library.

## Building

### Step 1
Get the source code and switch into the project's root directory:
```
git clone https://github.com/vobst/linpmem-cli
cd linpmem-cli
```

### Step 2 (optional)
If you have not done so already, start by installing the Rust programming language on your system. Instructions can be found [here](https://www.rust-lang.org/tools/install).

In case you already use Rust via Rustup, great, but make sure your installation is up-to-date by running
```
rustup update
```

### Step 3
Building `pmem` using Cargo is easy because the process is the same as for every other Rust program:
```
cargo build --release
```
This will generate a static binary located at `target/x86_64-unknown-linux-musl/release/pmem`.

## Installation

Likewise, installing can simply be done using:
```
cargo install --path . --locked
```
This command will install the `pmem` binary into Cargo's bin folder, e.g., `$HOME/.cargo/bin`.

## Uninstall

Oh, you don't like `pmem` :'(. Okay, to get rid of it just run:
```
cargo uninstall pmem
```
and remove the source directory:
```
rm -rf path/to/linpmem-cli
```

## Usage

`pmem` is a command-line client for the `linpmem` driver. Thus, you first have to [build the driver](). Assuming that you managed to successfully build the driver, load it with the `insmod` subcommand:
```
pmem insmod path/to/linpmem.ko
```
_Note: We are using a custom module loader, thus the system's `insmod` or `modprobe` binaries will not work._

Now, you can use `pmem` to interact with the driver:
```
$ pmem --help
Command-line client for the linpmem driver.

Small tool for loading and interacting with the linpmem driver. It lets you use the features of the driver in scripts and on the command line.

Usage: pmem [OPTIONS] [COMMAND]

Commands:
  insmod  Load the linpmem driver
  help    Print this message or the help of the given subcommand(s)

Options:
  -a, --address <ADDRESS>
          Address for physical read/write operations

  -v, --virt-address <VIRT_ADDRESS>
          Translate address in target process' address space (default: current process)

  -s, --size <SIZE>
          Size of buffer read operations

  -m, --mode <MODE>
          Access mode for read and write operations

          [possible values: byte, word, dword, qword, buffer]

  -w, --write <WRITE>
          Write the hex-encoded byte sequence

  -p, --pid <PID>
          Target process for cr3 info and virtual-to-physical translations

      --cr3
          Query cr3 value of target process (default: current process)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
By default, memory contents are written to stdout as raw bytes. Thus, you might want to use `xxd` to make them more human-friendly:
```
$ pmem --address 0x1ffe0040 -m buffer -s 16 | xxd
00000000: 4453 4454 7818 0000 0170 424f 4348 5320  DSDTx....pBOCHS
```

## Troubleshooting

At this point, a word of caution may be in order. Reading and writing arbitrary physical memory is considered dangerous. If you do not know what you are doing, DO NOT USE THIS TOOL.

For all the others, a good point to start may be the driver logs, simply:
```
cat /proc/kmsg | grep linpmem
```
They can be made more verbose by building the driver with `DEBUG` defined. If you come to the conclusion that the problem is with the `pmem` tool and not the driver, please open an issue.
