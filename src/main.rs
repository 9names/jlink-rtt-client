use memchr::memmem;
use std::io::{Read, Write};
use std::net::TcpStream;

// Update this to match what your tool says.
const END_OF_SEGGER_RTT_HEADER: &[u8; 28] = b"Process: JLinkGDBServerCLExe";

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
                let mut headerbuffer: [u8; 1024] = [0; 1024];
                loop {
                    // keep streaming until we find the end of the header
                    match stream.read(&mut buffer) {
                        Ok(size) => {
                            // make a slice out of what was read
                            let read = &buffer[0..size];

                            // need our slice sizes to match for copy_from_slices
                            // plus this will become the new "start of slice"
                            let end_of_target = read_so_far + size;
                            let target_slice = &mut headerbuffer[read_so_far..end_of_target];
                            target_slice.copy_from_slice(read);

                            // If we found our target string, we're ready to print whatever arrives over RTT
                            let mut found =
                                memmem::find_iter(&headerbuffer, END_OF_SEGGER_RTT_HEADER);
                            if found.next().is_some() {
                                break;
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
