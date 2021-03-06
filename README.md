![rpfm](https://user-images.githubusercontent.com/15714929/42595518-cd369b80-8552-11e8-8364-09d4ed2e42f6.JPG)
# Rusted PackFile Manager
***Rusted PackFile Manager*** (RPFM) is a... reimplementation in Rust and ***~~GTK3~~ Qt5*** of ***PackFile Manager*** (PFM), one of the best modding tools for Total War Games.

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

# Should I use this or stick to PFM?
It depends on what you need. Good things of RPFM are:
- **It has most of the features from PFM** and many of his own.
- **Is being actively developed**, so new features land from time to time.
- **Faster by a lot** in... more or less everything.
- **Far more stable** than PFM.
- **Every column in every table is actually decoded** (no more unknown columns).
- **UI similar to PFM**, so it's not hard to get used to when coming from PFM.

Bad things are:
- **You say it**.

# Status of the Project
RPFM currently supports **all Total War Games since Empire Total War**. Improvements are done every week, and I love cloudy days!

# Requirements (to use)
To use the ***Windows*** version, there are no requirements. Just extract it somewhere and execute it. After that, you need to go to settings, configure the paths of the games you have installed there. Otherwise, many features will not work. Also, if you want, use the ***Generate PAK File*** Special feature. It's explained in the manual.

In ***Linux*** make sure you have Qt5 installed. Then download and execute the program and configure it, just like the Windows version. That's all.

Also, the manual is [***HERE, READ IT BEFORE ASKING***][Manual].
# Requirements (to build)
To build this project, you need:
* ***Qt 5.8*** or higher.
* ***Rust toolchain*** (+1.26).
* ***Lzma lib*** (or equivalent).

In **Linux**, you just need to install the latest ***Qt5 package*** of your distro (at least *Qt 5.8*), and the DDS Plugin from *Qt5 ImageFormats* (you'll have to compile it, as it's no longer included by default in Qt). Also, you'll need the ***Rust Toolchain*** (at least +1.26, recommended using [***Rustup***][Rustup download]) and the LZMA lib.

In **Windows**, first you need to install the ***Rust Toolchain*** (at least +1.26, recommended using [***Rustup***][Rustup download]) the LZMA lib, and then install ***Qt5*** (at least *Qt 5.8*), and the DDS Plugin from *Qt5 ImageFormats* (you'll have to compile it, as it's no longer included by default in Qt). That's all.

To build, move to the repo directory and execute:
```bash
# For Windows
cargo run

# For Linux
cargo run
```

# Known Issues
- **Kailua throws useless errors!!!**: That feature is alpha and very experimental.
- **Linux paths doesn't work properly**: The linux version of some of the games have their files with a different structure than in Windows. To be fixed in another update.
- **Trying to extract a folder in a folder with a folder of the same name of the folder we are trying to extract gets you inside the folder instead of extracting your folder**: Weird behavior of the FileChooser. Still trying to find the cause.
- **My PackFile just vanished!!!!!**: If you open the same file with RPFM and other programs, and you have not ***DISABLED LAZY-LOADING IN THE SETTINGS***, this may happen. Not my fault, it's just how lazy-loading works. If you edit a file locked by another program, windows will give you the middle finger with this. So not RPFM's fault.

# FAQ
- **Why not helping with PFM instead of reimplementing it?**: because I wanted to learn a new language, and I already now a bit of C#. Also, where is the fun of that?
- **Why the code quality is not the very best?**: because I'm using this project to learn Rust, and I'm constantly rewriting code as I find new and better ways to write it.

# Credits
- ***Frodo45127***: I'm the guy who has made the program.
- ***Maruka*** (From Steam): He made the wazard hat's icon. Also, he helped with the research to decode RigidModel files.
- ***Mr. Jox*** (From Steam): He helped with A LOT of information about decoding RigidModels.
- ***Aexrael Dex*** (From Discord): He is who got all those suggested functions you see when editing a Lua Script.
- ***DrunkFlamingo*** (From Discord): He is who got all the Lua Types for Warhammer 2 so Kailua can work with WH2 scripts.
- ***Der Spaten*** (From Discord): He helped with the research to decode RigidModels, specially with the "Replace texture" functionality.
- ***Trolldemorted*** (From Github): He is who made all the research and code to get Arena PackFiles (and music PackedFiles in Rome 2 and Attila) decrypted and readable.

#### Special Thanks to:
- ***The guys that made PFM***: Most of the decoding stuff would have been very hard to work out without the PFM source code. So many thanks for make the PFM open source!
- ***The guys at CA***: They make good, easily-moddable games, and are very friendly with the community. Weird company in these times.

[Rustup download]: https://www.rustup.rs/ "Here you can download it :)"
[Patreon]: https://www.patreon.com/RPFM
[Manual]: https://frodo45127.github.io/rpfm/
