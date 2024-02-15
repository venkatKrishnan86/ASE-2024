# Assignment 1: Combfilter Implementation

**Name**: Venkatakrishnan V K

## Results
The audios which were tested are stored in the `audio` directory, and the resulting files are in `results` directory in their individual folder depending on whether IIR (`iir` folder) or FIR (`fir` folder) comb filter was used.

Python audios are also in the result folder in `python_iir` and `python_fir`

## Tests

In total, **38** tests have been written. The corresponding tests for the questions are as follows -
1. **FIR: Output is zero if input freq matches feedforward**
- Test *passed*
- Function written in `combiflter.rs`
- Function: `zero_output_input_freq_matching_feedforward()`


2. **IIR: amount of magnitude increase/decrease if input freq matches feedback**
- Test *passed*, checks for no overflow in i16 variables since amplitude exponentially increases on IIR feeding back with the same frequency
- Function written in `combiflter.rs`
- Function: `iir_magnitude_increase_for_input_freq_matching_feedforward()`


3. **FIR/IIR: correct result for VARYING input block size**

Varying block sizes test is written for *mono*, *stereo* as well as *spatial* (5 channel) audio. Test functions are in `combfilter.rs` -
- **IIR**: 
    - *Mono*: `different_buffer_sizes_mono_iir_test()`
    - *Stereo*: `different_buffer_sizes_stereo_iir_test()`
    - *Spatial*: `different_buffer_sizes_spatial_iir_test()`
- **FIR**: 
    - *Mono*: `different_buffer_sizes_mono_fir_test()`
    - *Stereo*: `different_buffer_sizes_stereo_fir_test()`
    - *Spatial*: `different_buffer_sizes_spatial_fir_test()`

4. **FIR/IIR: correct processing for zero input signal**

Test functions are in `combfilter.rs` -
- **IIR**: `zero_input_multi_channel_signal_test_iir()`
- **FIR**: `zero_input_multi_channel_signal_test_fir()`

5. **At least one more additional MEANINGFUL test to verify your filter implementation**

The rest of the unit tests are in `combfilter.rs` and `utils/mod.rs`