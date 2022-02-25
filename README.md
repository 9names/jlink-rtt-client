A tool for getting RTT data back from an embedded target via RTT from Segger's JLink tools.
This currently uses the socket that the JLinkGDBServer provides (which Segger refer to as telnet), but other methods of data acquisition are in-scope for this tool.
If your probe+chip combo is supported by probe-rs you probably want to use that instead.