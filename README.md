<p align="center"><img src="assets/icon.png" width="128" alt="PAF Classic icon"></p>

# PAF Classic

[![Build](https://github.com/dgmcint/paf/actions/workflows/release.yml/badge.svg)](https://github.com/dgmcint/paf/actions/workflows/release.yml)

An offline genealogy program for macOS, Windows and Linux, modelled on the way
Personal Ancestral File (PAF 5) worked: the Family and Pedigree screens, the
Individual list, editing windows, searches, calculators, printable reports and
GEDCOM import and export. It can also read PAF `.paf` family files.

PAF Classic is an independent, open-source project. It isn't affiliated with or endorsed by
FamilySearch or The Church of Jesus Christ of Latter-day Saints, and it doesn't
contain any of PAF's code or artwork.

![Family screen](shots/en.png)

## Download

Go to the [Releases page](https://github.com/dgmcint/paf/releases/latest) and pick the file for your computer:

| Computer | File |
| --- | --- |
| Mac with Apple Silicon (M1 and later) | `PAF-Classic-macos-apple-silicon.zip` |
| Mac with Intel processor | `PAF-Classic-macos-intel.zip` |
| Windows 10 or 11 (64-bit) | `PAF-Classic-windows-x64.zip` |
| Linux (64-bit) | `PAF-Classic-linux-x64.tar.gz` |

**macOS:** unzip, drag *PAF Classic* to Applications, then right-click it and
choose **Open** the first time. The app isn't notarized by Apple, so macOS asks
before opening it.

**Windows:** unzip and run `PAF Classic.exe`. If SmartScreen warns you, choose
**More info → Run anyway**.

**Linux:** unpack and run `./paf-classic`. Chinese, Japanese and Korean names
need a CJK font such as `fonts-noto-cjk`.

## Build it yourself

You need [Rust](https://rustup.rs) 1.85 or newer.

```sh
git clone https://github.com/dgmcint/paf.git
cd paf
cargo run --release
```

That builds and starts the program. The finished program is in `target/release/`.

On Linux, install these first (Debian/Ubuntu names):

```sh
sudo apt install libgtk-3-dev libxkbcommon-dev libwayland-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### Building for both kinds of Mac

```sh
rustup target add x86_64-apple-darwin aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin   # Apple Silicon
cargo build --release --target x86_64-apple-darwin    # Intel
```

## Making a release

The workflow in `.github/workflows/release.yml` builds all four versions on
GitHub every time you push. To publish downloads, tag a version:

```sh
git tag v0.1.0
git push origin v0.1.0
```

GitHub then builds the four packages and attaches them to a new release.

## Languages

Menus and windows come in English, German, Japanese, Korean, Swedish and
Chinese (Traditional), under **View → Language**.

## Where settings are kept

- macOS: `~/Library/Application Support/PAF Classic`
- Windows: `%APPDATA%\PAF Classic`
- Linux: `~/.config/PAF Classic`

Family files are saved wherever you choose, with the `.pafc` (PAF Classic file) extension.
Files saved by earlier versions with the `.fkf` extension still open. PAF Classic
also opens PAF 5 `.paf` files and GEDCOM `.ged` files; save them as `.pafc` to keep
your changes.

## Credit

Personal Ancestral File (PAF) was created by The Church of Jesus Christ of
Latter-day Saints and offered free to everyone for decades. PAF Classic exists
to keep the way that program worked available on today's computers, and the
credit for its design belongs to the Church and the people who built PAF.

## License

PAF Classic is released into the public domain under [the Unlicense](LICENSE).
Anyone may use, copy, change or share it, for any purpose, without asking.
