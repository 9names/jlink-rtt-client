use memchr::memmem;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    match TcpStream::connect("localhost:19021") {
        Ok(mut stream) => {
            // Do some config.
            // these should be run-time options, oh well
            // defmt uses channel 0, my RTT block is at 0x20000000
            let msg = b"$$SEGGER_TELNET_ConfigStr=RTTCh;0;SetRTTAddr;0x20000000$$!";
            stream.write_all(msg).unwrap();
            let mut buffer: [u8; 1024] = [0; 1024];

            // Putting this in a scope to dispose of it when done
            // Probably overkill, the header isn't that large.
            {
                let mut read_so_far = 0;
                // Small buffer to hold all of the header read so far, so we don't miss our
                // target string if it gets split over two reads
                // Make it a Vec so we can grow it if it needs to be larger
                let mut headerbuffer: Vec<u8> = Vec::with_capacity(2048);
                let mut start_of_final_line = None;
                loop {
                    // keep streaming until we find the end of the header
                    match stream.read(&mut buffer) {
                        Ok(size) => {
                            // make a slice out of what was read
                            let read = &buffer[0..size];

                            // need our slice sizes to match for copy_from_slices
                            // plus this will become the new "start of slice"
                            let end_of_target = read_so_far + size;
                            // ensure headbuffer is large enough to accept this
                            if headerbuffer.len() < end_of_target {
                                headerbuffer.resize(end_of_target, 0);
                            }
                            let target_slice = &mut headerbuffer[read_so_far..end_of_target];
                            target_slice.copy_from_slice(read);

                            // The last line of the RTT header is which program is exposing the RTT channel
                            // If we found our target string, we now need to wait until end of line
                            if start_of_final_line.is_none() {
                                start_of_final_line =
                                    memmem::find_iter(&headerbuffer, b"Process: ").next();
                            }

                            if let Some(idx) = start_of_final_line {
                                // If we found our target string, we're now looking for end of line instead.
                                let end_of_final_line =
                                    memmem::find_iter(&headerbuffer[idx..], "\n").next();
                                if let Some(end_idx) = end_of_final_line {
                                    let program = String::from_utf8_lossy(
                                        &headerbuffer[idx..(idx + end_idx)],
                                    );
                                    eprintln!("Attaching to rtt from {}", program);
                                    break;
                                }
                            }

                            // update our target buffer
                            read_so_far = end_of_target;
                        }
                        Err(_) => todo!(),
                    }
                }
            }

            // We finished reading the header - now just pump everything to stdout
            loop {
                match stream.read(&mut buffer) {
                    Ok(size) => {
                        let read = &buffer[0..size];
                        std::io::stdout().write_all(read).unwrap();
                        // If there's not much data throughput we won't see anything
                        // so flush constantly to keep us up-to-date
                        let _ = std::io::stdout().flush();
                    }
                    Err(_) => todo!(),
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
