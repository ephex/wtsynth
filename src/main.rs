// main.rs

use rodio::{OutputStream, source::Source};
use std::collections::HashMap;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use midir::Ignore;
use midir::MidiInput;

use crate::note::Note;
use crate::wtoscillator::WavetableOscillator;
use crate::wtoscillator::{WAVE_TYPE_SAW, WAVE_TYPE_SINE, WAVE_TYPE_SQUARE, WAVE_TYPE_TRI};

pub mod filter;
pub mod note;
pub mod wtoscillator;

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    //let mut voices1: HashMap<u8, NoteData> = HashMap::with_capacity(16);
    //let mut voices2: HashMap<u8, NoteData> = HashMap::with_capacity(16);
    let mut voices1: HashMap<u8, Note> = HashMap::with_capacity(16);
    let mut voices2: HashMap<u8, Note> = HashMap::with_capacity(16);
    //let voices: [usize; 16] = core::array::from_fn(|i| i+1);

    let mut midi_in= MidiInput::new("midi_read_fx")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("invalid input port selected")?
        }
    };

    // Start a quiet wave just to kick start the audio subsystem.
    let note = Note::new(
        WAVE_TYPE_SINE,
        440.0 * 2_f32.powf(69.0/12.0),
        0.0
    );
    // End the note.
    note.stop();

    println!("\nOpening connection");
    // Connection needs to be named to be kept alive.
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |stamp, message, _| {
            println!("{}: {:?} (len = {})", stamp, message, message.len());
            match message.len() {
                2 => {
                    // Aftertouch?
                }
                3 => {
                    // Regular note data.
                    if message[0] == 0x80 || (message[0] == 0x90 && message[2] == 0) {
                        // Note off.
                        let mut note = voices1.remove(&message[1]).ok_or("No note found!?").unwrap();
                        note.stop();
                        let mut note = voices2.remove(&message[1]).ok_or("No note found!?").unwrap();
                        note.stop();
                    }
                    else if message[0] == 0x90 {
                        // Note on w/ velocity.
                        let note_on = Arc::new(Mutex::new(true));
                        let note = Note::new(
                            WAVE_TYPE_TRI,
                            440.0 * 2_f32.powf((message[1] as f32 - 69.0)/12.0),
                            message[2] as f32 / 127.0
                        );
                        voices1.insert(message[1], note);
                        // Second voice sub-octave.
                        let note_on = Arc::new(Mutex::new(true));
                        let note = Note::new(
                            WAVE_TYPE_SQUARE,
                            440.0 * 2_f32.powf((message[1] as f32 - 69.0 - 12.0)/12.0),
                            message[2] as f32 / 127.0
                        );
                        voices2.insert(message[1], note);
                    }
                }
                _ => {
                    // Do nothing?
                }
            }
        },
        (),
    )?;

    //println!("Connection open, reading input from '{}'  (press enter to exit)", in_port_name);
    println!("Connection open (press enter to exit)");
    input.clear();
    stdin().read_line(&mut input)?; // Wait for enter/key press.
    
    println!("Closing connection");
    Ok(())
}