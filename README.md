# pmem

`pmem` is a small tool for loading and interacting with the [linpmem driver](https://github.com/velocidex/linpmem). It lets you use the features of the driver in scripts and on the command line. At the same time, this repository also provides a library that can be used by other programs that want to interface with the driver. The command-line application is simply a thin wrapper around this library.

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
You will also need the `x86_64-unknown-linux-musl` target. It can be installed by running:
```
rustup target add x86_64-unknown-linux-musl
```

### Step 3
Building `pmem` using Cargo is easy because the process is the same as for every other Rust program:
```
cargo build --release
```
This will generate two static binaries located at `target/x86_64-unknown-linux-musl/release/`:
- `pmem`: The fully-featured command-line client.
- `loader`: A smaller program that contains only the functionality needed to load and unload the driver.

## Installation

Likewise, installing can simply be done using:
```
cargo install --path . --locked
```
This command will install the `pmem` and `loader` binaries into Cargo's bin folder, e.g., `$HOME/.cargo/bin`.

Note: This will install the programs for the _current_ user, which is hopefully not the root user. In case you experience any troubles when running them through `sudo` or in a root shell, remember to add the binaries to root's PATH.

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

`pmem` is a command-line client for the `linpmem` driver. Thus, you first have to [build the driver](https://github.com/velocidex/linpmem#building). Assuming that you managed to successfully build the driver, load it with the `insmod` subcommand:
```
pmem insmod path/to/linpmem.ko
```
or the stand-alone loader
```
loader path/to/linpmem.ko
```

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
          Address for physical read operations

  -v, --virt-address <VIRT_ADDRESS>
          Translate address in target process' address space (default: current process)

  -s, --size <SIZE>
          Size of buffer read operations

  -m, --mode <MODE>
          Access mode for read operations

          [possible values: byte, word, dword, qword, buffer]

  -p, --pid <PID>
          Target process for cr3 info and virtual-to-physical translations

      --cr3
          Query cr3 value of target process (default: current process)

      --verbose
          Display debug output

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```
By default, memory contents are written to stdout as raw bytes. Thus, you might want to use `xxd` to make them more human-friendly:
```
# echo 1 > /proc/sys/kernel/kptr_restrict
$ sudo cat /proc/kallsyms | grep ' linux_banner$'
ffffffff9823bf20 D linux_banner
$ pmem -v 0xffffffff9823bf20
0x000000070923bf20
$ pmem -a 0x000000070923bf20 -m buffer -s 0x1000 | xxd
00000000: 4c69 6e75 7820 7665 7273 696f 6e20 362e  Linux version 6.
00000010: 342e 3131 2d68 6172 6465 6e65 6431 2d31  4.11-hardened1-1
00000020: 2d68 6172 6465 6e65 6420 286c 696e 7578  -hardened (linux
00000030: 2d68 6172 6465 6e65 6440 6172 6368 6c69  -hardened@archli
00000040: 6e75 7829 2028 6763 6320 2847 4343 2920  nux) (gcc (GCC)
00000050: 3133 2e32 2e31 2032 3032 3330 3830 312c  13.2.1 20230801,
00000060: 2047 4e55 206c 6420 2847 4e55 2042 696e   GNU ld (GNU Bin
00000070: 7574 696c 7329 2032 2e34 312e 3029 2023  utils) 2.41.0) #
00000080: 3120 534d 5020 5052 4545 4d50 545f 4459  1 SMP PREEMPT_DY
00000090: 4e41 4d49 4320 5475 652c 2032 3220 4175  NAMIC Tue, 22 Au
000000a0: 6720 3230 3233 2031 393a 3234 3a31 3920  g 2023 19:24:19
000000b0: 2b30 3030 300a 0000 81c9 0200 0000 0000  +0000...........
000000c0: 0b41 a578 65f5 70f2 63b0 d013 0941 ff70  .A.xe.p.c....A.p
000000d0: f2e9 b093 7274 0841 63b0 5f3b fca4 f40d  ....rt.Ac._;....
```

## Library
You can also use this project as a library to integrate the linpmem driver into your own applications. Currently we offer a public interface to Rust and C/C++. We might also offer a Python interface in the future (let me know if you are interested).

### C/C++
The normal build process also generates a static C library `libpmem.a` as well as a header files `libpmem.h[pp]` in `target/x86_64-unknown-linux-musl/release/`. Consult the header files for documentation of the libraries public C/C++ API. You can find example C programs in `examples/c`. To build the examples, simply type `make` when inside this directory.

## Troubleshooting

At this point, a word of caution may be in order. Reading arbitrary physical memory is considered dangerous. If you do not know what you are doing, DO NOT USE THIS TOOL.

For all the others, a good point to start debugging may be taking a look at the driver logs, simply:
```
sudo journalctl --since today -g linpmem
```
They can be made more verbose by building the driver with `DEBUG` defined. The user-space tools will also display debug output when being run with the `--verbose` flag.

If you come to the conclusion that the problem is with the `pmem` tool and not the driver, please open an issue.
