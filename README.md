Medusa Engine

Emergency Release Notice:
Due to increasing civil clampdowns in the United States, this project has been made open-source ahead of schedule. Documentation is minimal and will be filled out as long as possible, but here’s what you need to know:
Overview

The Medusa Engine reads any ephemeris file, extracts planetary and angle positions, and bitpacks them into a single f64—storing 8 bytes per planet. 

**    First run:** It is highly recommended to first run the engine on the ephemerides already included with the Swiss Ephemeris, sampling at 1 position per day density.
     For reasons unknown, subsequent runs will take less than 10 milliseconds, sometimes less than a millisecond, even if storing positions at 1 second density.
      
   ** Highly extensible:** Can easily be altered for sidereal calculations, making it ideal for Vedic astrology applications.

**Zenith Kernel Advantages**

The Zenith Kernel offers significant improvements over traditional temporary ephemerides:

    Extreme Compression – 8 bytes per planet, regardless of timespan, with all data stored in just 8 bytes.
    Precalculation – Every position is precalculated to the second, requiring zero runtime math.
    Cache Efficiency – Data fits within L1 cache, allowing instant access to all positions and speeds, across all time.
    Universal Little-Endian Format – Eliminates the need to compile the Swiss Ephemeris C code, which is notoriously problematic for Windows developers.

Example Implementation

bin.parabola-db serves as a fully functional, user-friendly parser demonstrating the engine’s capabilities.
Why This Matters

Medusa is a fully open-source, MIT-licensed astrological engine designed to democratize access to high-precision astrology. It runs flawlessly on the most low-powered hardware, and since it returns simple binary values, integrating it into any language or GUI is trivial.

MIT License means: Go ham, brother.
