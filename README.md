Medusa Engine

Emergency Release Notice:
Due to increasing civil clampdowns in the United States, this project has been made open-source ahead of schedule. Documentation is minimal and will be completed and updated for as long as possible, but here’s what you need to know:
Overview

The Medusa Engine reads any ephemeris file, extracts planetary and angle positions, and bitpacks them into a single f64—storing 8 bytes per planet. 

**    First run:** It is highly recommended to first run the engine on the ephemerides already included with the Swiss Ephemeris, sampling at 1 position per day density.
     For reasons unknown, subsequent runs will take less than 10 milliseconds, sometimes less than a millisecond, even if storing positions at 1 second density, and even if using an entirely different ephemeris file!
     After the first run, to store 30,000 years of data by the second in the same size kernel, download this file into your project folder: https://ssd.jpl.nasa.gov/ftp/eph/planets/Linux/de441/linux_m13000p17000.441 ,
     rename it to de441.eph, and run bin.medusajpl. Adding additional planets should be trivial simply by adding more swisseph constants to the bodies list, and changing the array size. Only 8 additional bytes will be added to the kernel size for each          tracked body.  

      
   ** Highly extensible:** Can easily be altered for sidereal calculations, making it ideal for Vedic astrology applications.

**Zenith Kernel Advantages**

The Zenith Kernel offers significant improvements over traditional temporary ephemerides:

    Extreme Compression – 8 bytes per planet, regardless of timespan, with all data stored in just 8 bytes.
    Precalculation – Every position is precalculated to the second, requiring zero runtime math.
    Cache Efficiency – Data fits within L1 cache, allowing instant access to all positions and speeds, across all time.
    Universal Little-Endian Format – Eliminates the need to compile the Swiss Ephemeris C code, which is notoriously problematic for Windows developers.

Example Implementation

bin.parabola-db serves as a fully functional, user-friendly parser demonstrating the accuracy of the kernel. If you like, you can simply use the kernel and parabola-db as is; simply cargo run --bin parabola-db [julian date or date in yyyy mm dd hh mm ss format]

To use in a PARABOLA fork or your own astrology/astronomy project, clone the repo:

git clone https://github.com/PARABOLAEngineering/Projectkronos.git && cd Projectkronos

To make your life a lot easier, I recommend using medusa_impactor.sh: 
chmod +x medusa_impactor.sh && ./medusa_impactor.sh 
This script will 
-clone the official swiss ephemeris repo at https://github.com/aloistr/swisseph.git 
-compile the swiss ephemeris into a static library
-generate Rust bindings using Bindgen and create the necessary files

From here, you should have everything you need to quickly create advanced astrology software for any platform. 

Why This Matters

Medusa is a fully open-source, MIT-licensed astrological engine designed to democratize access to high-precision astrology. It runs flawlessly on even the most low-powered hardware, and since it returns simple binary values, integrating it into any language or GUI is trivial.

MIT License means: Go ham, brother.
