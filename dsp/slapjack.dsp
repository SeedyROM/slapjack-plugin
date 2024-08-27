import("stdfaust.lib");

// Input stage with compression and harmonic distortion
input_stage = vgroup("1. Input", input_gain : compressor : distortion : output_gain)
with {
    input_gain = *(hslider("Input Gain[unit:dB]", 0, -12, 24, 0.1) : ba.db2linear : si.smoo);
    output_gain =*(hslider("Output Gain[unit:dB]", 0, -12, 24, 0.1) : ba.db2linear : si.smoo);
    
    compressor = hgroup("Compressor", co.compressor_mono(ratio, thresh, att, rel))
    with {
        ratio = hslider("Ratio", 4, 1, 20, 0.1);
        thresh = hslider("Threshold[unit:dB]", -10, -60, 0, 0.1);
        att = hslider("Attack[unit:ms]", 10, 1, 100, 1) / 1000;
        rel = hslider("Release[unit:ms]", 100, 10, 1000, 10) / 1000;
    };
    
    distortion = hgroup("Distortion", ef.cubicnl(drive, offset))
    with {
        drive = hslider("Drive", 0.1, 0, 1, 0.01);
        offset = hslider("Offset", 0, -1, 1, 0.01);
    };
};

slapback = _ <: (_, de.fdelay(maxdelay, delay)) :> *(1-wetdry), *(wetdry) : +
with {
    maxdelay = 44100; // Maximum delay in samples (1 second at 44.1kHz)
    
    delay = hslider("Time[unit:ms]", 100, 10, 500, 1) : *(ma.SR/1000) : si.smoo;
    wetdry = hslider("Mix", 0.5, 0, 1, 0.01);
    feedback = hslider("Feedback", 0, 0, 0.9, 0.01);
};

chorus_vibrato = _ <: *(1-mix), (de.fdelay(max_delay,delay_mod) * mix) :> _
with {
    max_delay = 2048; // Maximum delay in samples
    
    delay = hslider("Delay[unit:ms]", 10, 1, 20, 0.1) * ma.SR/1000 : si.smoo;
    rate = hslider("Rate[unit:Hz]", 2, 0.1, 10, 0.01) : si.smoo;
    depth = hslider("Depth", 0.15, 0, 1, 0.01) : si.smoo;
    mix = hslider("Mix", 0.5, 0, 1, 0.01);
    
    lfo = os.osc(rate) * depth;
    delay_mod = delay + (lfo * delay);
};

eff(N) = par(i, N, input_stage) : 
         par(i, N, vgroup("2. Chorus/Vibrato", chorus_vibrato)) : 
         par(i, N, vgroup("3. Slapback", slapback)) : 
         dm.freeverb_demo;

process = si.bus(2) : eff(2);