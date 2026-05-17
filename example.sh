#!/bin/bash

wsm="./rs-emhdr2asn1least.wasm"

chkasn1() {
	printf '%s' \
		30 \
		4e \
		801a4a2e20446f65203c6a2e646f65406578616d706c652e636f6d3e \
		811a4a2e20456f65203c6a2e656f65406578616d706c652e636f6d3e \
		820c68656c6c6f2c20776f726c64 \
		83080123456789abcdef |
		xxd -r -ps |
		xxd -ps |
		tr -d '\n' |
		python3 -m asn1tools convert -i der -o jer ./least.asn LeastHeaderInfo -
}

examplehdr() {
	#note: header keys are case sensitive
	printf '%s\n' \
		'From: J. Doe <j.doe@example.com>' \
		'To: J. Eoe <j.eoe@example.com>' \
		'Subject: Test' \
		'Date: Sun, 02 Oct 2016 07:06:22 -0700 (PDT)'
}

header2asn1der() {
	cat /dev/stdin |
		wasmtime run "${wsm}"
}

der2jer() {
	cat /dev/stdin |
		xxd -ps |
		tr -d '\n' |
		python3 \
			-m asn1tools \
			convert \
			-i der \
			-o jer \
			least.asn \
			LeastHeaderInfo \
			-
}

jer2toml() {
	cat /dev/stdin |
		dasel query --in=json --out=toml
}

hdr2toml2bat() {
	examplehdr |
		header2asn1der |
		der2jer |
		jer2toml |
		bat --language=toml
}

hdr2jer2date2unixtime() {
	examplehdr |
		header2asn1der |
		der2jer |
		jq --raw-output .date |
		xxd -r -ps |
		python3 -c 'import sys; import struct; import datetime; import functools; import operator; functools.reduce(
			lambda state, f: f(state),
			[
				struct.Struct("<q").unpack,
				operator.itemgetter(0),
				functools.partial(
					datetime.datetime.fromtimestamp,
					tz=datetime.timezone.utc,
				),
				print,
			],
			sys.stdin.buffer.read(),
		)'
}

hdr2toml2bat
hdr2jer2date2unixtime
