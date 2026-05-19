use rodio::{Decoder, MixerDeviceSink, Player};
use std::fs::File;
use std::io::BufReader;

fn main() {
    let handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
    let player = rodio::Player::connect_new(&handle.mixer());
    
    // Create a dummy sine wave or beep just to test if rodio plays anything
    let source = rodio::source::SineWave::new(440.0)
        .take_duration(std::time::Duration::from_millis(500))
        .amplify(0.20);
        
    player.append(source);
    
    println!("Playing beep!");
    player.sleep_until_end();
}
