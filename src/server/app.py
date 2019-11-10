"""
Server application.
"""
# pylint: disable=wrong-import-order,wrong-import-position
from gevent import monkey
monkey.patch_all()

import typing as tp
import dataclasses
import logging
import socket
import time

import gevent

import RPi.GPIO as gpio  # pylint: disable=import-error

from .. import config
from ..common import types
from ..common import utils

utils.configure_logging(filename="/var/log/cat.hunter.log")
LOG = logging.getLogger('server')


class Engine:

    def __init__(self, rotation_pin: int, pwm_pin: int) -> None:
        self.rotation_pin = rotation_pin
        self.pwm_pin = pwm_pin

    def setup(self) -> None:
        gpio.setup(self.rotation_pin, gpio.OUT)
        gpio.setup(self.pwm_pin, gpio.OUT)

    def forward(self) -> None:
        gpio.output(self.rotation_pin, True)
        gpio.output(self.pwm_pin, False)

    def backward(self) -> None:
        gpio.output(self.rotation_pin, False)
        gpio.output(self.pwm_pin, True)

    def stop(self) -> None:
        gpio.output(self.rotation_pin, False)
        gpio.output(self.pwm_pin, False)


class Light:

    def __init__(self, light_pin: int) -> None:
        self._light_pin = light_pin
        self._enabled = False

    def enable(self) -> None:
        gpio.setmode(gpio.BCM)
        gpio.setup(self._light_pin, gpio.OUT)
        gpio.output(self._light_pin, gpio.HIGH)
        self._enabled = True

    def disable(self) -> None:
        gpio.output(self._light_pin, gpio.LOW)
        self._enabled = False

    def trigger(self) -> None:
        if self._enabled:
            self.disable()
        else:
            self.enable()


class State:

    class Endpoint:

        def __init__(self) -> None:
            self.value: bool = False

        def move(self) -> bool:
            if self.value is True:
                return False
            self.value = True
            return True

        def stop(self) -> bool:
            if self.value is False:
                return False
            self.value = False
            return True

        def __bool__(self) -> bool:
            return self.value

        def __str__(self) -> str:
            return str(self.value)

    def __init__(self) -> None:
        self.forward = self.Endpoint()
        self.backward = self.Endpoint()
        self.left = self.Endpoint()
        self.right = self.Endpoint()

    def stop(self) -> bool:
        return (
            self.forward.stop()
            and self.backward.stop()
            and self.left.stop()
            and self.right.stop()
        )

    def __str__(self) -> str:
        return (
            f"State(forward={self.forward}, backward={self.backward}, "
            f"left={self.left}, right={self.right})"
        )


class EnginesHandler:

    def __init__(self) -> None:
        self.left_engine = Engine(23, 22)
        self.right_engine = Engine(17, 27)

    def _init_gpio(self) -> None:
        gpio.setmode(gpio.BCM)
        self.left_engine.setup()
        self.right_engine.setup()

    def update(self, state: State) -> None:
        if (state.left and state.right) or (state.forward and state.backward):
            self.stop()
            LOG.error("Wrong state")

        if state.forward:
            if state.left:
                self.left_engine.forward()
                self.right_engine.stop()
            elif state.right:
                self.right_engine.forward()
                self.left_engine.stop()
            else:
                self.left_engine.forward()
                self.right_engine.forward()
        elif state.backward:
            if state.left:
                self.right_engine.backward()
                self.left_engine.stop()
            elif state.right:
                self.left_engine.backward()
                self.right_engine.stop()
            else:
                self.left_engine.backward()
                self.right_engine.backward()
        elif state.left:
            self.left_engine.forward()
            self.right_engine.backward()
        elif state.right:
            self.right_engine.forward()
            self.left_engine.backward()
        else:
            self.stop()

    def stop(self) -> None:
        self._init_gpio()
        self.left_engine.stop()
        self.right_engine.stop()


class SignalHandler:

    def __init__(self) -> None:
        self.engines = EnginesHandler()
        self.state = State()
        self.light = Light(18)
        self._last_event = time.monotonic()
        self._running = False

    def run_forever(self) -> None:
        self._running = True
        while self._running:
            gevent.time.sleep(1)
            if (time.monotonic() - self._last_event) > 1:
                self.engines.stop()

    def stop(self):
        self.engines.stop()
        self._running = False

    def __call__(self, signal: types.Signals) -> None:
        self._last_event = time.monotonic()

        event_map: tp.Dict[types.Signals, tp.Callable] = {
            types.Signals.move_forward: self.state.forward.move,
            types.Signals.stop_forward: self.state.forward.stop,
            types.Signals.move_backward: self.state.backward.move,
            types.Signals.stop_backward: self.state.backward.stop,
            types.Signals.move_left: self.state.left.move,
            types.Signals.stop_left: self.state.left.stop,
            types.Signals.move_right: self.state.right.move,
            types.Signals.stop_right: self.state.right.stop,
            types.Signals.stop: self.state.stop,

            types.Signals.enable_light: self.light.enable,
            types.Signals.disable_light: self.light.disable,
            types.Signals.trigger_light: self.light.trigger,
        }
        if signal not in event_map:
            LOG.warning("Unknown signal: %s", signal)
            return

        LOG.debug("State: %s", self.state)
        LOG.debug("Signal: %s", signal.name)
        state_change = event_map[signal]()
        if state_change:
            self.engines.update(self.state)


def main() -> None:
    LOG.info("TCP server up and listening on 0.0.0.0:%s", config.SERVER_CTRL_PORT)
    sock = socket.socket(family=socket.AF_INET, type=socket.SOCK_STREAM)
    sock.bind(('0.0.0.0', config.SERVER_CTRL_PORT))
    sock.listen(1)

    signal_handler = SignalHandler()
    signal_handler.stop()
    gevent.spawn(signal_handler.run_forever)

    try:
        while True:
            connection, address = sock.accept()
            try:
                while True:
                    message = connection.recv(config.BUFFER_SIZE)
                    signal_code = int.from_bytes(message, byteorder='big')
                    LOG.info("Received %s from %s:%s", message, address[0], address[1])
                    try:
                        signal = types.Signals(signal_code)
                        signal_handler(signal)
                        result = types.SignalResult.ok
                    except ValueError:
                        result = types.SignalResult.error
                    connection.sendall(bytes([result]) + str(signal_handler.state).encode())
            except ConnectionError:
                LOG.error("connection is broken, wait for new..")
            finally:
                connection.close()
        return
        while True:
            address = 0, 0
            message = sock.recv(config.BUFFER_SIZE)
            signal_code = int.from_bytes(message, byteorder='big')
            LOG.info("Received %s from %s:%s", message, address[0], address[1])
            try:
                signal = types.Signals(signal_code)
                signal_handler(signal)
                result = types.SignalResult.ok
            except ValueError:
                result = types.SignalResult.error
            sock.sendto(bytes([result]) + str(signal_handler.state).encode(), address)
    except KeyboardInterrupt:
        LOG.info("Exiting...")
        signal_handler.stop()
    finally:
        gpio.cleanup()


if __name__ == '__main__':
    try:
        main()
    except Exception as err:
        LOG.exception(err)
