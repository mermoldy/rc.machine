import enum


class Signals(int, enum.Enum):

    # engine signals
    move_forward = 1
    move_backward = 2
    move_left = 3
    move_right = 4
    stop_forward = 5
    stop_backward = 6
    stop_left = 7
    stop_right = 8
    stop = 20

    # light signals
    enable_light = 21
    disable_light = 22
    trigger_light = 23


class SignalResult(int, enum.Enum):
    ok = 0
    error = 1
