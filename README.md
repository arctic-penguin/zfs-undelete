# zfs-undelete
an easy-to-use cli tool to recover files from zfs snapshots

## Usage

Use `zfs-undelete <file-to-restore>`. Works for file and folders.

Example:
```zsh
$ pwd
/home/penguin/screenshots
$ ls
screen.png  screen01.png  screen02.png
$ rm screen01.png
$ ls 
screen.png  screen02.png
$ zfs-undelete screen01.png
found file here:
"/home/.zfs/snapshot/znap_2022-11-14-0730_weekly/penguin/screenshots/screen01.png"
Restore file? [y/N] y
$ ls 
screen.png  screen01.png  screen02.png
```

## Installation

with cargo:
```zsh
$ cargo install zfs-undelete
```

from source:
```zsh
$ git clone https://github.com/arctic-penguin/zfs-undelete
$ cd zfs-undelete
$ cargo install --path .
```

## Dependencies
Requires the `findmnt` binary, which should be available on most systems.

## How does it work?
If the provided file path is located under a zfs dataset, `zfs-undelete` searches all snapshots of the dataset in reverse alphabetical order for the file.
It will restore the first file it finds.

Reverse alphabetical order is equivalent to reverse chronological order (newest first) for snapshots from most auto-snapshot tools.
