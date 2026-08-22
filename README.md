# sdr.rs
A comprehensive and widely compatible cross-platform SDR suite, written in Rust

some references to start for the first phase of research ingestion (ANALYZE THE ENTIRE CODEBASE OF EACH, WRITE A REPORT OF THE FUNCTIONALITY OF EACH, CITING WHICH COMPONENTS PERFORM WHAT ACTIONS AND UNDER WHAT CIRCUMSTANCES): 

- https://github.com/gnuradio/gnuradio
- https://github.com/AlexandreRouma/SDRPlusPlus
- https://blinry.org/50-things-with-sdr/ (I know this is for an RTL but it's just for some ideas on capabilities along with everything else here)
- https://www.sdrstore.eu/pluto-plus-sdr-setup-guide-sdrangel-gnu-radio-ethernet/
- https://github.com/NVlabs/cuda-oxide
- https://github.com/gnss-sdr/gnss-sdr
- https://github.com/cupy/cupy
- https://github.com/UPennEoR/MiniRadioTelescope
- https://github.com/jgromes/RadioLib
- https://github.com/greatscottgadgets/hackrf
- https://github.com/jopohl/urh
- https://github.com/analysiscenter/radio
- https://github.com/Hamlib/Hamlib
- https://github.com/achael/eht-imaging
- https://github.com/rpp0/gr-lora
- https://github.com/AlexMalov/RadioSniffer
- https://github.com/magicbug/Cloudlog (self-hosted amateur-radio logbook; JSON API — `/api/radio` for live CAT frequency/mode and `/api/qso` for ADIF contact logging — the station-logging integration target for this suite)


For testing (in the process): We have a Pluto + sdr unit + various antennas at our disposal that we can hook up to the computer when ready. 
