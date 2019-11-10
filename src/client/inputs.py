# pylint: disable=c-extension-no-member
"""
Inputs.
"""
import dataclasses
import logging
import typing as tp

import hid

from ..common import utils

utils.configure_logging()


LOG = logging.getLogger('inputs')


@dataclasses.dataclass
class Device:
    name: str
    vendor_id: int
    product_id: int

    def listen(self) -> tp.Generator[tuple, None, None]:
        """Open a device.
        """
        LOG.info("Opening %s (vid=%d, pid=%d)", self.name, self.vendor_id, self.product_id)
        device = hid.device()
        try:
            device.open(self.vendor_id, self.product_id)
        except IOError:
            raise IOError(f"Failed to open {self}") from None
        device.set_nonblocking(0)
        try:
            while True:
                data: tuple = device.read(64)
                if data:
                    yield data
                else:
                    break
        finally:
            device.close()


def get_devices() -> tp.Dict[str, Device]:
    return {
        d['product_string']: Device(
            name=d['product_string'],
            vendor_id=int(d['vendor_id']),
            product_id=int(d['product_id']))
        for d in hid.enumerate()
    }


def main():
    gamepad = get_devices().get('Wireless Controller')
    if not gamepad:
        raise Exception("Wireless Controller not found")
    for event in gamepad.listen():
        print(event)


main()

class InputBackend:

    def events(self) -> tp.Tuple[int, ...]:
        pass


class InputEvent:
    pass


class DualShock4Input(InputBackend):
    pass


class KeyboardInput(InputBackend):
    pass


def input_events() -> tp.Generator[InputEvent, None, None]:
    pass
