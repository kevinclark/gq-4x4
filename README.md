# GQ-4x4 Programming

The True USB Willem GQ-4x v4 (GQ-4x4) is a Univesral Programmer. Unfortunately
the driver and utility to use it is Windows only. This is an attempt to reverse
engineer the protocol.

## The How

I'm running the MCUMall supplied software in a windows 10 vm and using
wireshark to sniff the connection. On OS X I do this by bringing up the usb
bridge via: `sudo ifconfig XHC20 up` and then recording using wireshark 

A recording of the initialization phase, from device connection through the
MCUMall software setting initial configuration, is in Notes/initialization.pcapng.
The recording may also include firmware version verification.

Most communication appears to be happening via "quick commands" - control transfers
that send a vendor specific request (160) and a payload. The command and payload show
up in separate frames which can be isolated with tshark:

```
$ tshark -r docs/initialization.pcapng -Y "usb.setup.bRequest == 160 or usb.control.Response" \
         -T fields -E header=y -e frame.number -e usb.setup.wValue -e usb.control.Response > out.txt
```

## Status

Initial handshake appears to work.
