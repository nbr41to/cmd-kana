#!/usr/bin/env python3
"""Replay synthetic inputs against an isolated, locally installed Mozc server.

Uses the published Mozc 2.29 commands.proto and Unix IPC protocol, with only
Python's standard library. No user input is recorded and no system IME settings
are changed. This is an engine check, not a macOS Google Japanese Input UI test.
"""

import argparse
import contextlib
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import time


def varint(value):
    out = bytearray()
    while value >= 128:
        out.append((value & 127) | 128)
        value >>= 7
    out.append(value)
    return bytes(out)


def integer(field, value):
    return varint(field << 3) + varint(value)


def message(field, value):
    return varint(field << 3 | 2) + varint(len(value)) + value


def decode(data):
    position = 0

    def number():
        nonlocal position
        value = shift = 0
        while True:
            b = data[position]
            position += 1
            value |= (b & 127) << shift
            if b < 128:
                return value
            shift += 7

    def fields():
        nonlocal position
        result = {}
        while position < len(data):
            tag = number()
            field, wire = tag >> 3, tag & 7
            if wire == 4:
                break
            if wire == 0:
                value = number()
            elif wire == 2:
                size = number()
                value = data[position:position + size]
                position += size
            elif wire == 3:
                value = fields()
            elif wire in (1, 5):
                size = 8 if wire == 1 else 4
                value = data[position:position + size]
                position += size
            else:
                raise ValueError(wire)
            result.setdefault(field, []).append(value)
        return result

    return fields()


def first(fields, key, default=None):
    return fields.get(key, [default])[0]


def summarize(output):
    result = decode(first(output, 4, b''))
    preedit = decode(first(output, 5, b''))
    status = decode(first(output, 13, b''))
    return {
        'result': first(result, 2, b'').decode(),
        'preedit': ''.join(first(segment, 4, b'').decode() for segment in preedit.get(2, [])),
        'consumed': first(output, 3),
        'mode': first(output, 2),
        'activated': first(status, 1),
        'error': first(output, 11, 0),
    }


class Mozc:
    def __init__(self, server, profile):
        self.server = Path(server).resolve()
        self.profile = Path(profile)
        self.profile.mkdir(parents=True, exist_ok=True)
        self.log = (self.profile / 'server.log').open('w')
        self.process = subprocess.Popen(
            [str(self.server)],
            env=dict(os.environ, XDG_CONFIG_HOME=str(self.profile)),
            stdout=self.log,
            stderr=self.log,
        )
        for _ in range(50):
            try:
                metadata = decode((self.profile / 'mozc/.session.ipc').read_bytes())
                key = next(
                    values[0] for values in metadata.values()
                    if isinstance(values[0], bytes) and len(values[0]) == 32
                )
                self.version = next(
                    values[0].decode() for values in metadata.values()
                    if isinstance(values[0], bytes) and values[0].count(b'.') == 3
                )
                self.address = b'\0tmp/.mozc.' + key + b'.session'
                self.call(integer(1, 14))
                break
            except (OSError, StopIteration):
                time.sleep(0.05)
        else:
            raise RuntimeError('Mozc server did not become ready')

    def call(self, data):
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.settimeout(5)
            client.connect(self.address)
            pid, uid, _ = struct.unpack(
                '3i', client.getsockopt(socket.SOL_SOCKET, socket.SO_PEERCRED, 12)
            )
            if uid != os.getuid() or Path(f'/proc/{pid}/exe').resolve() != self.server:
                raise RuntimeError('Unexpected Mozc peer')
            client.sendall(data)
            client.shutdown(socket.SHUT_WR)
            response = bytearray()
            while block := client.recv(65536):
                response.extend(block)
            output = decode(response)
            if first(output, 11, 0):
                raise RuntimeError(f'Mozc session error: {output}')
            return output

    def create(self):
        return first(self.call(integer(1, 1)), 1)

    def key(self, session, keycode=None, special=None, modifiers=None):
        key = integer(1, keycode) if keycode is not None else integer(3, special)
        if modifiers is not None:
            key += integer(4, modifiers)
        return summarize(self.call(integer(1, 3) + integer(2, session) + message(3, key)))

    def delete(self, session):
        self.call(integer(1, 2) + integer(2, session))

    def command(self, session, command, mode=None):
        payload = integer(1, command)
        if mode is not None:
            payload += integer(3, mode)
        return summarize(self.call(integer(1, 5) + integer(2, session) + message(4, payload)))

    def configure(self, shift):
        # ROMAN input, KOTOERI keymap, specified Shift behavior.
        config = integer(40, 0) + integer(41, 3) + integer(59, shift)
        self.call(integer(1, 7) + message(5, config))

    def close(self):
        with contextlib.suppress(OSError):
            self.call(integer(1, 9))
        self.process.poll()  # Reap the daemon launcher if it has exited.
        self.log.close()


def type_text(mozc, session, text):
    committed = ''
    output = None
    for character in text:
        # The macOS key translator sends uppercase keycodes, with no separate
        # Shift modifier, when only Shift is held with a printable character.
        output = mozc.key(session, keycode=ord(character))
        committed += output['result']
        if not output['consumed']:
            committed += character
    return committed, output


def native_eisu_commit(mozc, session):
    # mac/GoogleJapaneseInputController.mm maps the first Eisu on marked text
    # to SWITCH_INPUT_MODE(HALF_ASCII), and the native double tap to
    # COMMIT_RAW_TEXT. A further Eisu must not commit duplicate text.
    switched = mozc.command(session, 5, mode=3)
    assert not switched['result'], switched
    raw = mozc.command(session, 19)
    assert not raw['preedit'], raw
    repeated = mozc.command(session, 19)
    assert not repeated['result'] and not repeated['preedit'], repeated
    return raw['result']


def check(server, profile):
    fixture = Path(__file__).resolve().parents[1] / 'tests/fixtures/mozc-composition.tsv'
    rows = [line.split('\t') for line in fixture.read_text().splitlines()]
    mozc = Mozc(server, profile)
    try:
        mozc.configure(shift=1)
        for roman, expected in rows:
            session = mozc.create()
            mozc.key(session, special=2)
            committed, _ = type_text(mozc, session, roman)
            output = mozc.key(session, special=3)
            assert committed + output['result'] == expected, (roman, expected, committed, output)
            assert not output['preedit'], (roman, output)
            mozc.delete(session)

            session = mozc.create()
            mozc.key(session, special=2)
            committed, _ = type_text(mozc, session, roman)
            assert not committed, (roman, committed)
            assert native_eisu_commit(mozc, session) == roman, roman
            mozc.delete(session)

        mode_cases = []
        for shift in [0, 1, 2]:
            mozc.configure(shift)
            for mode in [1, 2, 5]:  # Hiragana, Katakana, half-width Katakana.
                for roman in ['Ka', 'Kya', 'KANJI', 'aKa', 'gakkou', 'nnya', 'kfa']:
                    session = mozc.create()
                    mozc.key(session, special=2)
                    mozc.command(session, 5, mode)
                    committed, output = type_text(mozc, session, roman)
                    assert not committed, (shift, mode, roman, committed)
                    preedit = output['preedit']
                    result = native_eisu_commit(mozc, session)
                    assert result == roman, (shift, mode, roman, result)
                    mode_cases.append(dict(
                        shift=shift, mode=mode, roman=roman, preedit=preedit, result=result
                    ))
                    mozc.delete(session)
        return dict(
            engine=mozc.version,
            composition_cases=len(rows),
            native_commit_cases=len(rows),
            mode_cases=mode_cases,
        )
    finally:
        mozc.close()


def main():
    if not __debug__:
        raise SystemExit('Run without -O: this verification script requires assertions')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--server', default='/usr/lib/mozc/mozc_server', help='Path to the local Linux Mozc server')
    parser.add_argument('--report', type=Path, help='Write synthetic mode cases and case counts as JSON')
    args = parser.parse_args()
    if not Path(args.server).is_file():
        parser.error('Mozc server is missing; install mozc-server or pass --server')
    if (Path.home() / '.mozc').is_dir():
        parser.error('Legacy ~/.mozc would override the isolated test profile; use a clean Linux test account')
    with tempfile.TemporaryDirectory(prefix='cmd-kana-mozc-') as profile:
        result = check(args.server, profile)
    if args.report:
        args.report.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    print(f"Mozc {result['engine']}: {result['composition_cases']} compositions, "
          f"{result['native_commit_cases']} native commits, {len(result['mode_cases'])} mode cases passed")


if __name__ == '__main__':
    main()
