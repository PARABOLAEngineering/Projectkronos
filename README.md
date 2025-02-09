🚨 Medusa Engine - Emergency Open Source Release 🚨

Due to increasing civil clampdowns in the United States, Medusa Engine has been made open-source ahead of schedule. While documentation is still being developed, here’s what you need to know to get started:
🔍 Overview

Medusa Engine is a high-performance astrological computation tool that reads any ephemeris file, extracts planetary and angle positions, and bitpacks them into a single f64, storing 8 bytes per planet.
⚡ Usage

cargo run --bin <chosen engine> [start date - end date]

First Run:

    Start with the included Swiss Ephemeris files, sampling at 1 position per day.
    For reasons yet unknown, subsequent runs will complete in <10ms, even with second-level precision!
    To store 30,000 years of second-by-second planetary data in the same kernel size:
        Download: de441.eph
        Rename it to de441.eph
        Run:

    cargo run --bin medusajpl

    Adding planets? Just expand the bodies list—only 8 extra bytes per body!

🛠️ Highly Extensible

Easily adaptable for sidereal calculations, making it ideal for Vedic astrology applications.
🔥 Why Medusa? - The Zenith Kernel Advantage

Unlike traditional ephemerides, Zenith Kernel offers:
✅ Extreme Compression – 8 bytes per planet, no matter the timespan.
✅ Zero Runtime Math – Every position is precomputed to the second.
✅ L1 Cache Efficiency – Instant access to all planetary positions & speeds.
✅ Universal Little-Endian Format – No more Swiss Ephemeris C compilation nightmares.
📌 Example Implementation

A working parser, bin.parabola-db, validates Medusa's accuracy.

cargo run --bin parabola-db Julian date, or yyyy mm dd hh mm ss

🛠️ Quickstart: Integrate Medusa into Your Project

git clone https://github.com/PARABOLAEngineering/ProjectKronos.git && cd ProjectKronos

To automate setup, run:

chmod +x medusa_impactor.sh && ./medusa_impactor.sh

then:
cargo build 

This script:
✅ Clones Swiss Ephemeris (https://github.com/aloistr/swisseph.git)
✅ Compiles it into a static library
✅ Generates Rust bindings with Bindgen
🚀 Why This Matters

Medusa Engine is an MIT-licensed, fully open-source astrological computation engine.

    Runs efficiently on low-powered hardware
    Returns simple binary values for seamless cross-platform integration
    Democratizes access to high-precision astrology
