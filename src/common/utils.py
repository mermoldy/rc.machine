import logging
import sys

LOGGING_FORMAT = '%(asctime)s | %(levelname)-7s | %(name)s: %(message)s'


def configure_logging(level=None, filename=None) -> None:
    kwargs = {}
    if filename:
        kwargs['filename'] = filename
        kwargs['filemode'] = 'a'
    else:
        kwargs['stream'] = sys.stderr
    logging.basicConfig(
        format=LOGGING_FORMAT,
        level=level or logging.DEBUG,
        **kwargs)
