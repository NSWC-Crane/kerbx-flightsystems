# KerbX Flight Systems

An avionics and mission planning computer for the Rapid Kerbal Delivery System. This
project leverages Kerbal Space Program and KRPC to help teach programming concepts
for embedded safety-critical systems.

## Workspace Elements

## Requirements

### Installing Bazelisk
Bazelisk is the preferred method to install the bazel build system. Bazel is the build system required by krpc.

To install bazelisk you will require npm: ``sudo apt install npm``

The following instructions will install bazelisk via npm locally without the
requirement of root privileges.
1. ``mkdir $HOME/.npm-packages``
2. ``npm config set prefic $HOME/.npm-packages``
3. Add the following to your .bashrc (or equivalent for your shell):
```buildoutcfg
NPM_PACKAGES=$HOME/.npm-packages
export PATH="${PATH}:$NPM_PACKAGES/bin"
export MANPATH=${MANPATH}:$NPM_PACKAGES/share/man
```
4. Open a new terminal and type: ``npm install -g @bazel/bazelisk``

Bazel/Bazelisk will now be installed as the local user.

Refs: 
1. https://github.com/sindresorhus/guides/blob/main/npm-global-without-sudo.md
2. https://docs.bazel.build/versions/4.1.0/install-bazelisk.html
3. https://github.com/bazelbuild/bazelisk/blob/master/README.md

### KRPC
You can obtain the version of KRPC used with kerbx flight systems here:
https://github.com/drwhomphd/krpc

Prior to starting the bazel build you will need to set two symlinks. First is a symlink to Kerbal
Space Program downloaded from Steam and the second is a symlink to where your
mono libraries are.
1. Go to the directory where you cloned krpc with git
2. ``ln -s $HOME/.steam/debian-installation/steamapps/common/Kerbal\ Space\ Program $PWD/lib/ksp``
3. ``ln -s /usr/lib/mono/4.5 $PWD/lib/mono-4.5``

You can then execute tools/install.sh to build and install KRPC:
``bash tools/install.sh``

Refs:
1. https://krpc.github.io/krpc/compiling.html

## Compilation Instructions

First you will need to install rustup in order to manage rust. Instructions for installing Rust for
your platform can be found here: https://www.rust-lang.org/learn/get-started

We suggest using `rustup` over your distributions package manager due to the
quicker availability of updated compiler version, however package manager versions should work just
as well.

Building kerbx flight systems is as easy as typing ``cargo build`` in the project directory.

To run kerbx flight systems you can launch either the avionics binary or the flight planner without
needing to install either:
- This will display the help for the avionics binary: ``cargo run --bin avionics -- -h``
- This will display the help for the flight planner binary: ``cargo run --bin flightplanner -- -h``