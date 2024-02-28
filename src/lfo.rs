use std::f32::consts::PI;

pub fn lfo(mod_freq: f32, sample_index: f32) -> f32 {
    (mod_freq * 2.0 * PI * sample_index).sin()
}

#[cfg(test)]
mod tests {
    
}