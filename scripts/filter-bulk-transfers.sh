#!/usr/bin/env bash

tshark -r docs/initialization.pcapng -Y "usb.darwin.endpoint_type == 2 and usb.endpoint_address.number == 1 and usb.darwin.request_type == 1" -w bulk_transfers.pcapng
editcap -C 30: bulk_transfers.pcapng trimmed.pcapng

