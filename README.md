Parts: as this was an emergency publication due to increasing civil clampdowns by authorities in the united states, documentation is minimal but here's an overview: 
the medusa engine is meant to read any ephemeris file given, read the positions for every designated planet and angle, and bitpack into a single f64: 8 bytes per planets. 
first run can take around an hour; subsequent runs using the same technique, even if using a different module, will take less than 10 milliseconds. 
this framework is highly extensible and will be able to implant sidereal calculations for vedic astrology apps effortlessly.
the zenith kernel provides advantages over traditional temporary ephemerides through: 
1 extreme compression (8 bytes per planet, for any number of positions and any timespan, all in 8 bytes) 
2 precalculation (all values are precalculated by the second and packed in readable form with no runtime math required) 
3 the size and method of reading allow for persistent storage in system caches such as the l1 cache, for instant access to all positions and speeds, for all time)
4 universal little-endian format (bypasses the need for compiling the C code of the swiss ephemeris, which is highly problematic for Windows-based devs)
bin.parabola-db is an excellent example of a fully functioning, user-friendly parser. 
in conclusion, this is a fully open-source, MIT licensed (meaning: go ham brother) astrological engine meant to democratize the space by running flawlessly on even the lowest-powered hardware. 
since only simple binary values are returned, hooking into any language and GUI should be trivial using the knowledge here. 
