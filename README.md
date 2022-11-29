# zfs-undelete
an easy-to-use cli tool to recover files from zfs snapshots

## Usage

Use `zfs-undelete <file-to-restore>`. Works for file and folders.

By default, it restores the last version it can find.

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

Interactive mode:
```zsh
$ pwd
/home/penguin/rust/zfs-undelete
$ ls README.md
ls: cannot access 'README.md': No such file or directory
$ zfs-undelete -V README.md
0: /home/.zfs/snapshot/znap_2022-11-17-1315_frequent, 1132 kB
1: /home/.zfs/snapshot/znap_2022-11-14-0742_weekly, 75 B
choose [0-1]: 0
$ ls README.md
README.md
```


## Installation

### using rust
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

### Arch Linux
```zsh
$ paru -S zfs-undelete
```
or
```zsh
$ paru -S zfs-undelete-git
```

## How does it work?
If the provided file path is located under a zfs dataset, `zfs-undelete` searches all snapshots of the dataset in reverse alphabetical order for the file.
It will restore the first file it finds.

Reverse alphabetical order is equivalent to reverse chronological order (newest first) for snapshots from most auto-snapshot tools.
