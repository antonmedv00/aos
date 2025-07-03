use anyhow::{anyhow, Result};
use pc_keyboard::{ScancodeSet, ScancodeSet1, EventDecoder, HandleControl, DecodedKey};
use pc_keyboard::layouts::Us104Key;
use x86_64::instructions::port::Port;

pub struct Keyboard {
    scancode_set:  ScancodeSet1,
    event_decoder: EventDecoder<Us104Key>,
    status_port:   Port<u8>,
    data_port:     Port<u8>
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            scancode_set:  ScancodeSet1::new(),
            event_decoder: EventDecoder::new(Us104Key, HandleControl::Ignore),
            status_port:   Port::new(0x64),
            data_port:     Port::new(0x60)
        }
    }

    pub fn read_char(&mut self) -> Result<Option<char>> {
        let data = unsafe {
            let status = self.status_port.read();
            if status & 1 == 0 { return Ok(None); }

            self.data_port.read()
        };

        let Some(event) = self.scancode_set.advance_state(data).map_err(|e| anyhow!("{e:?}"))? else {
            return Ok(None);
        };
        
        Ok(
            self.event_decoder
                .process_keyevent(event)
                .and_then(|x| {
                    match x {
                        DecodedKey::Unicode(x) => Some(x),
                        _ => None
                    }
                })
        )
    }
}
