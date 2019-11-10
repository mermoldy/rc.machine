"""
Client application.
"""
# pylint: disable=no-member

import logging
import socket
import typing as tp

import pygame

from .. import config
from ..common import types
from ..common import utils

utils.configure_logging()


LOG = logging.getLogger('client')

EVENT_TO_SIGNAL_MAP: tp.Dict[tp.Tuple[int, int], types.Signals] = {
    (pygame.KEYDOWN, pygame.K_UP): types.Signals.move_forward,
    (pygame.KEYUP, pygame.K_UP): types.Signals.stop_forward,
    (pygame.KEYDOWN, pygame.K_DOWN): types.Signals.move_backward,
    (pygame.KEYUP, pygame.K_DOWN): types.Signals.stop_backward,
    (pygame.KEYDOWN, pygame.K_LEFT): types.Signals.move_left,
    (pygame.KEYUP, pygame.K_LEFT): types.Signals.stop_left,
    (pygame.KEYDOWN, pygame.K_RIGHT): types.Signals.move_right,
    (pygame.KEYUP, pygame.K_RIGHT): types.Signals.stop_right,

    (pygame.KEYDOWN, pygame.K_l): types.Signals.trigger_light,
}


class SignalHandler:

    def __init__(self, server_addr: str, server_port: int) -> None:
        self._server_addr = server_addr
        self._server_port = server_port
        self._socket = socket.socket(family=socket.AF_INET, type=socket.SOCK_STREAM)
        self._socket.settimeout(1)
        try:
            self._socket.connect((self._server_addr, self._server_port))
        except ConnectionError as err:
            raise ConnectionError(f"Cannot connecto to TCP server by {self._server_addr}:{self._server_port}") from None

    def send(self, signal: types.Signals) -> tp.Optional[str]:
        LOG.info("Sending '%s' signal ...", signal.name)
        try:
            self._socket.sendall(bytes([signal.value]))
            received_message = self._socket.recv(config.BUFFER_SIZE)
        except socket.timeout:
            LOG.error(
                "Failed to process '%s' signal due to socket timeout (%s:%s).",
                signal.name,
                self._server_addr,
                self._server_port)
            return None

        try:
            code, body = types.SignalResult.ok, ""
            #result = received_message[0]
            #code, body = int(result[0]), result[1:].decode()
            #code = types.SignalResult(code)
        except (KeyError, ValueError) as err:
            LOG.error("Failed to parse response: %s", err)

        if code == types.SignalResult.ok:
            LOG.info("Result: %s", body)
        if code == types.SignalResult.error:
            LOG.error("Result: %s", body)
        return body

    def __call__(self, events: tp.List) -> tp.Optional[str]:
        state = None
        for event in events:
            if event.type not in (pygame.KEYDOWN, pygame.KEYUP):
                continue
            event_pair = (event.type, event.key)
            signal = EVENT_TO_SIGNAL_MAP.get(event_pair)
            if signal:
                state = self.send(signal)
            else:
                LOG.warning("Unknown event: %s", event_pair)
        return state
