"""
Application entrypoint.
"""
# pylint: disable=wrong-import-order,wrong-import-position
from gevent import monkey
monkey.patch_all()

import gevent
import logging

import pygame
from .. import config
from ..common import types
from ..common import utils
from . import camera
from . import controls

utils.configure_logging()


LOG = logging.getLogger('client')


def main() -> None:
    pygame.init()  # pylint: disable=no-member
    pygame.key.set_repeat(100, 100)
    pygame.display.set_caption("Cat.Hunter")
    screen = pygame.display.set_mode((720, 576))
    pygame._state = ''

    camera_stream = camera.CameraStream(config.SERVER_ADDRESS, config.SERVER_CAMERA_PORT)
    signal_handler = controls.SignalHandler(config.SERVER_ADDRESS, config.SERVER_CTRL_PORT)

    font = pygame.font.Font(None, 22)

    def rander_camera():
        while True:
            image = next(camera_stream)  # type: ignore
            if image:
                screen.blit(image, (0, 0))
                #textsurface = font.render(pygame._state, True, (0, 0, 0))
                #screen.blit(textsurface, (0, 0))
                pygame.display.flip()
            gevent.idle(0.01)

    def handle_events():
        while True:
            events = pygame.event.get()
            for event in events:
                 # pylint: disable=no-member
                if (
                    (event.type == pygame.QUIT)
                    or (event.type == pygame.KEYDOWN and event.key in (
                        pygame.K_ESCAPE,
                        pygame.K_q,
                    ))
                ):
                    raise SystemExit()
            state = signal_handler(events)
            # if state:
            #    pygame._state = state

            gevent.idle(0.01)

    try:
        gevent.joinall([
            gevent.spawn(rander_camera),
            gevent.spawn(handle_events)
        ])
    finally:
        signal_handler.send(types.Signals.stop)
        LOG.info("Exiting...")


if __name__ == '__main__':
    main()
