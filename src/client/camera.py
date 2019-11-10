import io
import typing as tp
import urllib.request

import pygame
from PIL import Image as pil_image


class CameraStream:

    def __init__(self, server_addr: str, server_port: int) -> None:
        self._stream = urllib.request.urlopen(f"http://{server_addr}:{server_port}")
        self._buffer = b''
        self._resolution = (720, 576)

    def __next__(self) -> tp.Optional[pygame.Surface]:
        self._buffer += self._stream.read(1024)
        header = self._buffer.find(b'\xff\xd8')
        body = self._buffer.find(b'\xff\xd9')
        if header != -1 and body != -1:
            data = self._buffer[header: body + 2]
            self._buffer = self._buffer[body + 2:]
        else:
            return None

        image = pil_image.open(io.BytesIO(data))
        image = pygame.image.frombuffer(image.tobytes(), self._resolution, "RGB")
        return image
